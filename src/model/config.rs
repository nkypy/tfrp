/*
 * Copyright (c) 2020  [Jack Shih <i@kshih.com>]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "lowercase", deserialize = "lowercase"))]
pub enum Protocol {
    TCP,
    UDP,
    HTTP,
    HTTPS,
}

// server config
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub common: ServerConfigCommon,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfigCommon {
    pub bind_port: u16,
    pub auth_token: String,
}

// client config
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub common: ClientConfigCommon,
    pub clients: HashMap<String, ProxyClientConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfigCommon {
    pub server_addr: String,
    pub server_port: u16,
    // pub log_level: String,
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyClientConfig {
    pub protocol: Protocol,
    pub local_port: u16,
    pub remote_port: u16,
}
