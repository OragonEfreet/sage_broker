use crate::BrokerSettings;
use async_std::sync::Arc;
use sage_mqtt::ReasonCode;

/// The main broker object
#[derive(Default, Debug, Clone)]
pub struct Broker {
    /// Settings used for this broker
    pub settings: Arc<BrokerSettings>,
}

impl TryFrom<BrokerSettings> for Broker {
    type Error = ReasonCode;

    fn try_from(settings: BrokerSettings) -> Result<Self, Self::Error> {
        if settings.is_valid() {
            Ok(Broker {
                settings: Arc::new(settings),
                ..Default::default()
            })
        } else {
            Err(ReasonCode::ImplementationSpecificError)
        }
    }
}
