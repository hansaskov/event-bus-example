use crate::{event_bus::{Event, EventKind}, module::{Module, ModuleCtx}};
use anyhow::{Ok, Result};
use opcua::{
    client::{Client, ClientBuilder, DataChangeCallback, IdentityToken, Session},
    crypto::SecurityPolicy,
    types::{
        DataValue, EndpointDescription, MessageSecurityMode, MonitoredItemCreateRequest, NodeId, TimestampsToReturn, UserTokenPolicy
    },
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub url: String,
    pub namespace: u16,
    pub variables: Vec<String>
}


pub struct OPCUA {
    ctx: ModuleCtx,
    client: Client,
    config: Config
}


impl OPCUA {
    pub fn new(ctx: ModuleCtx, config: Config) -> Self {
        let client = ClientBuilder::new()
            .application_name("Simple Client")
            .application_uri("urn:SimpleClient")
            .product_uri("urn:SimpleClient")
            .trust_server_certs(true)
            .create_sample_keypair(true)
            .session_retry_limit(3)
            .client()
            .unwrap();

        Self { ctx, client, config }
    }

    pub fn data_value_to_string(data_value: &DataValue) -> String {
        if let Some(ref value) = data_value.value {
            // Convert the variant value to a string representation
            format!("{:?}", value)
        } else if let Some(ref status) = data_value.status {
            // Return error status as string if value is not present
            format!("Error: {}", status)
        } else {
            "No value or status".to_string()
        }
    }

    pub async fn subscribe_to_variables(&self, session: Arc<Session>, ns: u16) -> Result<()> {
        // Creates a subscription with a data change callback
        let sender = self.ctx.sender.clone();
        let module_name = self.ctx.name.clone();

        let subscription_id = session
            .create_subscription(
                Duration::from_secs(1),
                10,
                30,
                0,
                0,
                true,
                DataChangeCallback::new(move |dv, item| {
                    println!("Data change from server:");
                    println!("Data value: {:?}", dv.value);
                    let value_string = if let Some(ref value) = dv.value {
                        format!("{:?}", value)
                    } else if let Some(ref status) = dv.status {
                        format!("Error: {}", status)
                    } else {
                        "No value or status".to_string()
                    };
                    println!("Data value as string: {}", value_string);
                    
                    // Create and send the event directly
                    let event = Event {
                        module: module_name.clone(),
                        inner: EventKind::Log(value_string),
                    };
                    
                    if let Err(e) = sender.send(event) {
                        eprintln!("Failed to send event: {}", e);
                    }
                }),
            )
            .await?;
        println!("Created a subscription with id = {}", subscription_id);
    
        // Create some monitored items
        let items_to_create: Vec<MonitoredItemCreateRequest> = self.config.variables
            .iter()
            .map(|v| NodeId::new(ns, v.clone()).into())
            .collect();
        let _ = session
            .create_monitored_items(subscription_id, TimestampsToReturn::Both, items_to_create)
            .await?;
    
        Ok(())
    }
}

impl Module for OPCUA {
    async fn run(&mut self) -> Result<()> {

        let endpoint: EndpointDescription = (
            self.config.url.as_str(),
            SecurityPolicy::None.to_str(),
            MessageSecurityMode::None,
            UserTokenPolicy::anonymous(),
        )
            .into();

        let (session, event_loop) = self.client.connect_to_matching_endpoint(endpoint, IdentityToken::Anonymous).await.unwrap();

        let handle = event_loop.spawn();
        session.wait_for_connection().await;

        if let Err(result) = self.subscribe_to_variables(session.clone(), self.config.namespace).await {
            println!(
                "ERROR: Got an error while subscribing to variables - {}",
                result
            );
            let _ = session.disconnect().await;
        }

        let session_c = session.clone();
        tokio::task::spawn(async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                println!("Failed to register CTRL-C handler: {e}");
                return;
            }
            let _ = session_c.disconnect().await;
        });

        handle.await.unwrap();
       
        Ok(())
    }
}
