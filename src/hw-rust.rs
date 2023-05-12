// ==================================================================================
//   Copyright (c) 2023 Abhijit Gadgil
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.
// ==================================================================================

use ric_subscriptions::models::{SubscriptionParams, SubscriptionParamsClientEndpoint};
use rmr::{RMRClient, RMRError, RMRMessageBuffer};
use rnib::entities::NbIdentity;
use xapp::XApp;

const RIC_HEALTH_CHECK_REQ: i32 = 100;
const RIC_HEALTH_CHECK_RES: i32 = 101;

fn handle_ric_health_check_request(
    msg: &mut RMRMessageBuffer,
    client: &RMRClient,
) -> Result<(), RMRError> {
    let reply = b"OK\n";
    let _ = msg.set_payload(reply);
    let _ = msg.set_mtype(RIC_HEALTH_CHECK_RES);

    client.rts_msg(msg).expect("Send to Sender Failed.");
    Ok(())
}

fn rmr_message_handler_noop(
    _msg: &mut RMRMessageBuffer,
    _client: &RMRClient,
) -> Result<(), RMRError> {
    Ok(())
}

// FIXME: Hard coded right now
const SUB_MGR_HOST: &'static str = "http://service-ricplt-submgr-http.ricplt:3800";
const SUBSCRIPTION_URL: &'static str = "ric/v1/subscriptions";

struct HwApp {
    xapp: XApp,
}

impl HwApp {
    fn send_subscription(&self, meid: &str) -> std::io::Result<()> {
        let client = SubscriptionParamsClientEndpoint {
            host: Some(String::from("service-ricxapp-hw-go-rmr.ricxapp")),
            http_port: Some(8080),
            rmr_port: Some(4560),
        };

        let sub_params = SubscriptionParams {
            client_endpoint: Box::new(client),
            meid: meid.to_string(),
            ran_function_id: 1,
            e2_subscription_directives: None,
            subscription_details: vec![],
            subscription_id: None,
        };

        let json = serde_json::to_string(&sub_params)?;

        let req_client = reqwest::blocking::Client::new();

        let path = format!("{}/{}", SUB_MGR_HOST, SUBSCRIPTION_URL);

        let response = req_client.post(path).body(json).send().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error sending request: {}", e),
            )
        })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error : {}", response.status()),
            ))
        }
    }

    fn get_nodeb_ids(&self) -> std::io::Result<Vec<NbIdentity>> {
        self.xapp.rnib_get_nodeb_ids().map_err(|e| e.into())
    }

    fn ready_fn(&self) -> std::io::Result<()> {
        log::info!("HwApp is Ready! Getting connected nodes and subscribing for notifications!");
        let nodebs = self.get_nodeb_ids()?;

        for nodeb in nodebs {
            log::info!(
                "Sending Subscription Request for Node: '{}",
                nodeb.inventory_name
            );
            self.send_subscription(&nodeb.inventory_name)?;
        }
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let env = env_logger::Env::default().filter_or("MY_LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    // TODO: Get it from the config
    let xapp = XApp::new("4560", RMRClient::RMRFL_NONE)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Xapp Init Error."))?;

    let mut hw_xapp = HwApp { xapp };

    let mut rmr_ready_wait_counter = 0;

    hw_xapp
        .xapp
        .register_handler(60000, rmr_message_handler_noop);

    hw_xapp
        .xapp
        .register_handler(RIC_HEALTH_CHECK_REQ, handle_ric_health_check_request);

    hw_xapp.xapp.start();

    loop {
        if !hw_xapp.xapp.is_rmr_ready() {
            std::thread::sleep(std::time::Duration::from_secs(1));
            rmr_ready_wait_counter += 1;
            if rmr_ready_wait_counter == 10 {
                log::error!("RMR Not Ready after 10 seconds! Stopping Xapp");
                hw_xapp.xapp.stop();
                break;
            }
        } else {
            if let Err(error) = hw_xapp.ready_fn() {
                log::error!("XApp Ready Function returned error: {}.", error);
                hw_xapp.xapp.stop();
            }

            break;
        }
    }

    log::info!("Xapp Ready. Waiting for RMR Messages to process!");

    hw_xapp.xapp.join();

    Ok(())
}
