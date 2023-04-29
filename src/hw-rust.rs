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

use std::time::Duration;

use rmr::{RMRClient, RMRError, RMRMessageBuffer};
use xapp::XApp;

fn rmr_message_handler_noop(
    msg: &mut RMRMessageBuffer,
    client: &RMRClient,
) -> Result<(), RMRError> {
    Ok(())
}

fn main() -> std::io::Result<()> {
    let env = env_logger::Env::default().filter_or("MY_LOG_LEVEL", "debug");
    env_logger::init_from_env(env);

    // TODO: Get it from the config
    let mut hw_xapp = XApp::new("4560", RMRClient::RMRFL_NONE)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Xapp Init Error."))?;

    hw_xapp.register_handler(60000, rmr_message_handler_noop);

    hw_xapp.start();

    eprintln!("{:#?}", std::env::vars().collect::<Vec<(String, String)>>());
    std::thread::sleep(Duration::from_secs(10));

    hw_xapp.stop();

    hw_xapp.join();

    Ok(())
}
