use crate::{BrokerSettings, Peer};
use sage_mqtt::{ReasonCode, SubAck, Subscribe};
use std::sync::Arc;

/// Simply returns a ConnAck package
/// With the correct packet identifier
/// List of all possible returned reason codes:
/// - Success: The subscription is accepted and the maximum QoS sent will be QoS 0. This might be a lower QoS than was requested.
/// - GrantedQoS1: The subscription is accepted and the maximum QoS sent will be QoS 1. This might be a lower QoS than was requested.
/// - GrantedQoS2: The subscription is accepted and any received QoS will be sent to this subscription.
/// - UnspecifiedError: The subscription is not accepted and the Server either does not wish to reveal the reason or none of the other Reason Codes apply.
/// - ImplementationSpecificError: The SUBSCRIBE is valid but the Server does not accept it.
/// - NotAuthorized: The Client is not authorized to make this subscription.
/// - TopicFilterInvalid: The Topic Filter is correctly formed but is not allowed for this Client.
/// - PacketIdentifierInUse: The specified Packet Identifier is already in use.
/// - QuotaExceeded: An implementation or administrative imposed limit has been exceeded.
/// - SharedSubscriptionsNotSupported: The Server does not support Shared Subscriptions for this Client.
/// + SubscriptionIdentifiersNotSupported: The Server does not support Subscription Identifiers; the subscription is not accepted.
/// - WildcardSubscriptionsNotSupported: The Server does not support Wildcard Subscriptions; the subscription is not accepted.
pub async fn run(
    settings: Arc<BrokerSettings>,
    packet: Subscribe,
    peer: Arc<Peer>,
) -> Result<(), ReasonCode> {
    // Take the client if exist, from the peer, and at it a new sub
    if let Some(session) = peer.session() {
        let mut suback = SubAck {
            packet_identifier: packet.packet_identifier,
            ..Default::default()
        };

        for (topic, options) in packet.subscriptions {
            // QoS Checking
            let mut reason_code = settings.check_qos(options.qos);

            if topic.share().is_some() {
                reason_code = ReasonCode::SharedSubscriptionsNotSupported;
            }

            if topic.has_wildcards() {
                reason_code = ReasonCode::WildcardSubscriptionsNotSupported;
            }

            suback.reason_codes.push(reason_code);
            if matches!(
                reason_code,
                ReasonCode::Success | ReasonCode::GrantedQoS1 | ReasonCode::GrantedQoS2
            ) {
                session
                    .subs()
                    .write()
                    .unwrap()
                    .add(topic, options, packet.subscription_identifier);
            }
        }
        peer.send(suback.into())
    } else {
        // If not session present, close the peer.
        // Send an UnspecifiedError error for each topic
        peer.send(
            SubAck {
                packet_identifier: packet.packet_identifier,
                reason_codes: vec![ReasonCode::UnspecifiedError; packet.subscriptions.len()],
                ..Default::default()
            }
            .into(),
        );
        peer.close();
    }
    Ok(())
}
