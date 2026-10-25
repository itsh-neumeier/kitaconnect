//! Current membership data only; no OrganizationAuthority or inferred MFA decision.
use crate::{IdentityId, login::LoginFacilityId};
use kitaconnect_service_transport::membership_http::MembershipHttpClient;
use serde::{
    Deserialize,
    de::{self, MapAccess, Visitor, value::MapAccessDeserializer},
};
use std::{
    fmt,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::{net::TcpStream, sync::Semaphore, time::timeout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadError {
    Rejected,
    Unavailable,
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rejected => "membership request rejected",
            Self::Unavailable => "membership check unavailable",
        })
    }
}
impl std::error::Error for ReadError {}

#[derive(PartialEq, Eq)]
pub struct MembershipSnapshot {
    identity: IdentityId,
    facility: LoginFacilityId,
    membership_version: u64,
    policy_version: u64,
    idle_seconds: u64,
    absolute_seconds: u64,
    step_up_seconds: u64,
}
impl fmt::Debug for MembershipSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MembershipSnapshot([REDACTED])")
    }
}
impl MembershipSnapshot {
    pub fn identity(&self) -> IdentityId {
        self.identity
    }
    pub fn facility(&self) -> LoginFacilityId {
        self.facility
    }
    pub fn membership_version(&self) -> u64 {
        self.membership_version
    }
    pub fn policy_version(&self) -> u64 {
        self.policy_version
    }
    pub fn idle_seconds(&self) -> u64 {
        self.idle_seconds
    }
    pub fn absolute_seconds(&self) -> u64 {
        self.absolute_seconds
    }
    pub fn step_up_seconds(&self) -> u64 {
        self.step_up_seconds
    }
}
#[derive(Clone, Copy)]
pub struct ReadLimits {
    pub parallel: usize,
    pub connect_time: Duration,
    pub total_time: Duration,
}
impl ReadLimits {
    fn validate(self) -> Result<(), ReadError> {
        if self.parallel == 0
            || self.parallel > Semaphore::MAX_PERMITS
            || self.connect_time > self.total_time
            || [self.connect_time, self.total_time]
                .iter()
                .any(|d| d.is_zero() || Instant::now().checked_add(*d).is_none())
        {
            return Err(ReadError::Unavailable);
        }
        Ok(())
    }
}
pub struct MembershipReader {
    transport: MembershipHttpClient,
    address: SocketAddr,
    limits: ReadLimits,
    active: Semaphore,
}
impl MembershipReader {
    pub fn new(
        transport: MembershipHttpClient,
        address: SocketAddr,
        limits: ReadLimits,
    ) -> Result<Self, ReadError> {
        limits.validate()?;
        Ok(Self {
            transport,
            address,
            active: Semaphore::new(limits.parallel),
            limits,
        })
    }
    pub async fn read(
        &self,
        token: &[u8; 32],
        identity: IdentityId,
        facility: LoginFacilityId,
    ) -> Result<Option<MembershipSnapshot>, ReadError> {
        let _permit = self
            .active
            .try_acquire()
            .map_err(|_| ReadError::Unavailable)?;
        timeout(self.limits.total_time, async {
            let socket = timeout(self.limits.connect_time, TcpStream::connect(self.address))
                .await
                .map_err(|_| ReadError::Unavailable)?
                .map_err(|_| ReadError::Unavailable)?;
            let response = self
                .transport
                .request(socket, token, identity.value(), facility.value())
                .await
                .map_err(|_| ReadError::Unavailable)?;
            match response.status() {
                200 => parse(response.body(), identity, facility),
                400 | 401 | 404 | 405 | 413 | 415 => Err(ReadError::Rejected),
                _ => Err(ReadError::Unavailable),
            }
        })
        .await
        .map_err(|_| ReadError::Unavailable)?
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    #[serde(deserialize_with = "membership")]
    membership: Option<Wire>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    identity_ref: String,
    facility_id: String,
    membership_version: String,
    policy_version: String,
    idle_seconds: u64,
    absolute_seconds: u64,
    step_up_seconds: u64,
}
fn membership<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Wire>, D::Error> {
    struct Object;
    impl<'de> Visitor<'de> for Object {
        type Value = Option<Wire>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("membership object or null")
        }
        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
            Wire::deserialize(MapAccessDeserializer::new(map)).map(Some)
        }
    }
    d.deserialize_any(Object)
}
fn number(text: &str) -> Result<u128, ReadError> {
    if text.len() > 39
        || !matches!(text.as_bytes().first(), Some(b'1'..=b'9'))
        || !text.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(ReadError::Unavailable);
    }
    text.parse().map_err(|_| ReadError::Unavailable)
}
fn version(text: &str) -> Result<u64, ReadError> {
    u64::try_from(number(text)?).map_err(|_| ReadError::Unavailable)
}
fn parse(
    body: &[u8],
    identity: IdentityId,
    facility: LoginFacilityId,
) -> Result<Option<MembershipSnapshot>, ReadError> {
    if body.iter().find(|b| !b.is_ascii_whitespace()) != Some(&b'{') {
        return Err(ReadError::Unavailable);
    }
    let envelope: Envelope = serde_json::from_slice(body).map_err(|_| ReadError::Unavailable)?;
    let Some(wire) = envelope.membership else {
        return Ok(None);
    };
    if number(&wire.identity_ref)? != identity.value()
        || number(&wire.facility_id)? != facility.value()
        || !(1..=900).contains(&wire.idle_seconds)
        || !(1..=28_800).contains(&wire.absolute_seconds)
        || !(1..=300).contains(&wire.step_up_seconds)
        || wire.idle_seconds > wire.absolute_seconds
        || wire.step_up_seconds > wire.absolute_seconds
    {
        return Err(ReadError::Unavailable);
    }
    Ok(Some(MembershipSnapshot {
        identity,
        facility,
        membership_version: version(&wire.membership_version)?,
        policy_version: version(&wire.policy_version)?,
        idle_seconds: wire.idle_seconds,
        absolute_seconds: wire.absolute_seconds,
        step_up_seconds: wire.step_up_seconds,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn ids() -> (IdentityId, LoginFacilityId) {
        (
            IdentityId::new(1).unwrap(),
            LoginFacilityId::new(2).unwrap(),
        )
    }
    fn valid() -> serde_json::Value {
        json!({"membership":{"identity_ref":"1","facility_id":"2","membership_version":"18446744073709551615","policy_version":"1","idle_seconds":900,"absolute_seconds":28800,"step_up_seconds":300}})
    }
    fn check(value: &serde_json::Value) -> Result<Option<MembershipSnapshot>, ReadError> {
        let (i, f) = ids();
        parse(&serde_json::to_vec(value).unwrap(), i, f)
    }
    #[test]
    fn current_snapshot_is_exact_and_null_never_selects_defaults() {
        let s = check(&valid()).unwrap().unwrap();
        assert_eq!((s.identity(), s.facility()), ids());
        assert_eq!((s.membership_version(), s.policy_version()), (u64::MAX, 1));
        assert_eq!(
            (s.idle_seconds(), s.absolute_seconds(), s.step_up_seconds()),
            (900, 28800, 300)
        );
        assert_eq!(format!("{s:?}"), "MembershipSnapshot([REDACTED])");
        assert!(check(&json!({"membership":null})).unwrap().is_none());
        assert!(check(&json!({})).is_err());
        let mut value = valid();
        value["membership"]["identity_ref"] = json!(u128::MAX.to_string());
        let s = parse(
            &serde_json::to_vec(&value).unwrap(),
            IdentityId::new(u128::MAX).unwrap(),
            ids().1,
        )
        .unwrap()
        .unwrap();
        assert_eq!(s.identity().value(), u128::MAX);
    }
    #[test]
    fn fields_types_duplicates_and_sequence_representations_are_rejected() {
        let (i, f) = ids();
        for text in [
            r#"null"#,
            r#"[null]"#,
            r#"{"membership":null,"membership":null}"#,
            r#"{"membership":null,"member\u0073hip":null}"#,
            r#"{"membership":null}{}"#,
            r#"{"membership":["1","2","1","1",900,28800,300]}"#,
            r#"{"membership":{"identity_ref":"1","identity\u005fref":"1","facility_id":"2","membership_version":"1","policy_version":"1","idle_seconds":900,"absolute_seconds":28800,"step_up_seconds":300}}"#,
        ] {
            assert!(parse(text.as_bytes(), i, f).is_err());
        }
        for field in [
            "identity_ref",
            "facility_id",
            "membership_version",
            "policy_version",
            "idle_seconds",
            "absolute_seconds",
            "step_up_seconds",
        ] {
            let mut value = valid();
            value["membership"].as_object_mut().unwrap().remove(field);
            assert!(check(&value).is_err());
            let mut value = valid();
            value["membership"][field] = serde_json::Value::Null;
            assert!(check(&value).is_err());
        }
        let mut v = valid();
        v["membership"]["mfa_required"] = json!(false);
        assert!(check(&v).is_err());
        let mut v = valid();
        v["extra"] = json!(false);
        assert!(check(&v).is_err());
    }
    #[test]
    fn identifiers_versions_and_duration_bounds_fail_closed() {
        for invalid in [
            "0",
            "01",
            "-1",
            "+1",
            "1.0",
            "1e0",
            " 1",
            "340282366920938463463374607431768211456",
        ] {
            assert!(number(invalid).is_err());
        }
        for field in ["identity_ref", "facility_id"] {
            let mut v = valid();
            v["membership"][field] = json!("3");
            assert!(check(&v).is_err());
        }
        for field in ["membership_version", "policy_version"] {
            for bad in ["0", "01", "18446744073709551616"] {
                let mut v = valid();
                v["membership"][field] = json!(bad);
                assert!(check(&v).is_err());
            }
        }
        for (field, max) in [
            ("idle_seconds", 900),
            ("absolute_seconds", 28800),
            ("step_up_seconds", 300),
        ] {
            for bad in [json!(0), json!(max + 1), json!("1"), json!(1.5), json!(-1)] {
                let mut v = valid();
                v["membership"][field] = bad;
                assert!(check(&v).is_err());
            }
        }
        let mut v = valid();
        v["membership"]["absolute_seconds"] = json!(100);
        assert!(check(&v).is_err());
    }
    #[test]
    fn outer_budget_and_deadlines_cannot_be_disabled() {
        let valid = ReadLimits {
            parallel: 1,
            connect_time: Duration::from_secs(1),
            total_time: Duration::from_secs(2),
        };
        assert!(valid.validate().is_ok());
        for parallel in [0, Semaphore::MAX_PERMITS + 1] {
            assert!(ReadLimits { parallel, ..valid }.validate().is_err());
        }
        for time in [Duration::ZERO, Duration::MAX] {
            assert!(
                ReadLimits {
                    connect_time: time,
                    ..valid
                }
                .validate()
                .is_err()
            );
            assert!(
                ReadLimits {
                    total_time: time,
                    ..valid
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            ReadLimits {
                connect_time: Duration::from_secs(3),
                ..valid
            }
            .validate()
            .is_err()
        );
    }
}
