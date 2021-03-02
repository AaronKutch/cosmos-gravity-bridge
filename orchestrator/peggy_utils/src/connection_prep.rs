//! This module provides useful tools for handling the Contact and Web30 connections for the relayer, orchestrator and various other utilities.
//! It's a common problem to have conflicts between ipv4 and ipv6 localhost and this module is first and foremost supposed to resolve that problem
//! by trying more than one thing to handle potentially misconfigured inputs.

use clarity::Address as EthAddress;
use contact::client::Contact;
use deep_space::address::Address as CosmosAddress;
use peggy_proto::peggy::query_client::QueryClient as PeggyQueryClient;
use peggy_proto::peggy::QueryDelegateKeysByEthAddress;
use peggy_proto::peggy::QueryDelegateKeysByOrchestratorAddress;
use std::process::exit;
use std::time::Duration;
use tokio::time::delay_for;
use tonic::transport::Channel;
use url::Url;
use web30::client::Web3;

pub struct Connections {
    pub web3: Option<Web3>,
    pub grpc: Option<PeggyQueryClient<Channel>>,
    pub contact: Option<Contact>,
}

/// Returns the three major RPC connections required for Gravity
/// operation in a error resilient manner. TODO find some way to generalize
/// this so that it's less ugly
pub async fn create_rpc_connections(
    grpc_url: Option<String>,
    legacy_rpc_url: Option<String>,
    eth_rpc_url: Option<String>,
    timeout: Duration,
) -> Connections {
    let mut web3 = None;
    let mut grpc = None;
    let mut contact = None;
    if let Some(grpc_url) = grpc_url {
        let url = Url::parse(&grpc_url)
            .unwrap_or_else(|_| panic!("Invalid Cosmos gRPC url {}", grpc_url));
        check_scheme(&url, &grpc_url);
        let cosmos_grpc_url = grpc_url.trim_end_matches('/').to_string();
        // try the base url first.
        let try_base = PeggyQueryClient::connect(cosmos_grpc_url).await;
        match try_base {
            // it worked, lets go!
            Ok(val) => grpc = Some(val),
            // did not work, now we check if it's localhost
            Err(e) => {
                warn!(
                    "Failed to access Cosmos gRPC with {:?} trying fallback options",
                    e
                );
                if grpc_url.to_lowercase().contains("localhost") {
                    let port = url.port().unwrap_or(80);
                    // this should be http or https
                    let prefix = url.scheme();
                    let ipv6_url = format!("{}://::1:{}", prefix, port);
                    let ipv4_url = format!("{}://127.0.0.1:{}", prefix, port);
                    let ipv6 = PeggyQueryClient::connect(ipv6_url.clone()).await;
                    let ipv4 = PeggyQueryClient::connect(ipv4_url.clone()).await;
                    warn!("Trying fallback urls {} {}", ipv6_url, ipv4_url);
                    match (ipv4, ipv6) {
                        (Ok(v), Err(_)) => {
                            info!("Url fallback succeeded, your cosmos gRPC url {} has been corrected to {}", grpc_url, ipv4_url);
                            grpc = Some(v)
                        },
                        (Err(_), Ok(v)) => {
                            info!("Url fallback succeeded, your cosmos gRPC url {} has been corrected to {}", grpc_url, ipv6_url);
                            grpc = Some(v)
                        },
                        (Ok(_), Ok(_)) => panic!("This should never happen? Why didn't things work the first time?"),
                        (Err(_), Err(_)) => panic!("Could not connect to Cosmos gRPC, are you sure it's running and on the specified port? {}", grpc_url)
                    }
                } else if url.port().is_none() || url.scheme() == "http" {
                    let body = url.host_str().unwrap_or_else(|| {
                        panic!("Cosmos gRPC url contains no host? {}", grpc_url)
                    });
                    // transparently upgrade to https if available, we can't transparently downgrade for obvious security reasons
                    let https_on_80_url = format!("https://{}:80", body);
                    let https_on_443_url = format!("https://{}:443", body);
                    let https_on_80 = PeggyQueryClient::connect(https_on_80_url.clone()).await;
                    let https_on_443 = PeggyQueryClient::connect(https_on_443_url.clone()).await;
                    warn!(
                        "Trying fallback urls {} {}",
                        https_on_443_url, https_on_80_url
                    );
                    match (https_on_80, https_on_443) {
                        (Ok(v), Err(_)) => {
                            info!("Https upgrade succeeded, your cosmos gRPC url {} has been corrected to {}", grpc_url, https_on_80_url);
                            grpc = Some(v)
                        },
                        (Err(_), Ok(v)) => {
                            info!("Https upgrade succeeded, your cosmos gRPC url {} has been corrected to {}", grpc_url, https_on_443_url);
                            grpc = Some(v)
                        },
                        (Ok(_), Ok(_)) => panic!("This should never happen? Why didn't things work the first time?"),
                        (Err(_), Err(_)) => panic!("Could not connect to Cosmos gRPC, are you sure it's running and on the specified port? {}", grpc_url)
                    }
                } else {
                    panic!("Could not connect to Cosmos gRPC! please check your grpc url {} for errors {:?}", grpc_url, e)
                }
            }
        }
    }
    if let Some(legacy_rpc_url) = legacy_rpc_url {
        let url = Url::parse(&legacy_rpc_url)
            .unwrap_or_else(|_| panic!("Invalid Cosmos legacy RPC url {}", legacy_rpc_url));
        check_scheme(&url, &legacy_rpc_url);
        let cosmos_legacy_url = legacy_rpc_url.trim_end_matches('/');
        let base_contact = Contact::new(&cosmos_legacy_url, timeout);
        let try_base = base_contact.get_syncing_status().await;
        match try_base {
            // it worked, lets go!
            Ok(_) => contact = Some(base_contact),
            // did not work, now we check if it's localhost
            Err(e) => {
                warn!(
                    "Failed to access Cosmos Leagcy RPC with {:?} trying fallback options",
                    e
                );
                if legacy_rpc_url.to_lowercase().contains("localhost") {
                    let port = url.port().unwrap_or(80);
                    // this should be http or https
                    let prefix = url.scheme();
                    let ipv6_url = format!("{}://::1:{}", prefix, port);
                    let ipv4_url = format!("{}://127.0.0.1:{}", prefix, port);
                    let ipv6_contact = Contact::new(&ipv6_url, timeout);
                    let ipv4_contact = Contact::new(&ipv4_url, timeout);
                    let ipv6_test = ipv6_contact.get_syncing_status().await;
                    let ipv4_test = ipv4_contact.get_syncing_status().await;
                    warn!("Trying fallback urls {} {}", ipv6_url, ipv4_url);
                    match (ipv4_test, ipv6_test) {
                        (Ok(_), Err(_)) => {
                            info!("Url fallback succeeded, your cosmos legacy rpc url {} has been corrected to {}", legacy_rpc_url, ipv4_url);
                            contact = Some(ipv4_contact)
                        },
                        (Err(_), Ok(_)) => {
                            info!("Url fallback succeeded, your cosmos legacy rpc url {} has been corrected to {}", legacy_rpc_url, ipv6_url);
                            contact = Some(ipv6_contact)
                        },
                        (Ok(_), Ok(_)) => panic!("This should never happen? Why didn't things work the first time?"),
                        (Err(_), Err(_)) => panic!("Could not connect to Cosmos legacy rpc, are you sure it's running and on the specified port? {}", legacy_rpc_url)
                    }
                } else if url.port().is_none() || url.scheme() == "http" {
                    let body = url.host_str().unwrap_or_else(|| {
                        panic!("Cosmos legacy rpc url contains no host? {}", legacy_rpc_url)
                    });
                    // transparently upgrade to https if available, we can't transparently downgrade for obvious security reasons
                    let https_on_80_url = format!("https://{}:80", body);
                    let https_on_443_url = format!("https://{}:443", body);
                    let https_on_80_contact = Contact::new(&https_on_80_url, timeout);
                    let https_on_443_contact = Contact::new(&https_on_443_url, timeout);
                    let https_on_80_test = https_on_80_contact.get_syncing_status().await;
                    let https_on_443_test = https_on_443_contact.get_syncing_status().await;
                    warn!(
                        "Trying fallback urls {} {}",
                        https_on_443_url, https_on_80_url
                    );
                    match (https_on_80_test, https_on_443_test) {
                        (Ok(_), Err(_)) => {
                            info!("Https upgrade succeeded, your cosmos legacy rpc url {} has been corrected to {}", legacy_rpc_url, https_on_80_url);
                            contact = Some(https_on_80_contact)
                        },
                        (Err(_), Ok(_)) => {
                            info!("Https upgrade succeeded, your cosmos legacy rpc url {} has been corrected to {}", legacy_rpc_url, https_on_443_url);
                            contact = Some(https_on_443_contact)
                        },
                        (Ok(_), Ok(_)) => panic!("This should never happen? Why didn't things work the first time?"),
                        (Err(_), Err(_)) => panic!("Could not connect to Cosmos legacy rpc, are you sure it's running and on the specified port? {}", legacy_rpc_url)
                    }
                } else {
                    panic!("Could not connect to Cosmos legacy rpc! please check your url {} for errors {:?}", legacy_rpc_url, e)
                }
            }
        }
    }
    if let Some(eth_rpc_url) = eth_rpc_url {
        let url = Url::parse(&eth_rpc_url)
            .unwrap_or_else(|_| panic!("Invalid Ethereum RPC url {}", eth_rpc_url));
        check_scheme(&url, &eth_rpc_url);
        let eth_url = eth_rpc_url.trim_end_matches('/');
        let base_web30 = Web3::new(&eth_url, timeout);
        let try_base = base_web30.eth_block_number().await;
        match try_base {
            // it worked, lets go!
            Ok(_) => web3 = Some(base_web30),
            // did not work, now we check if it's localhost
            Err(e) => {
                warn!(
                    "Failed to access Ethereum RPC with {:?} trying fallback options",
                    e
                );
                if eth_url.to_lowercase().contains("localhost") {
                    let port = url.port().unwrap_or(80);
                    // this should be http or https
                    let prefix = url.scheme();
                    let ipv6_url = format!("{}://::1:{}", prefix, port);
                    let ipv4_url = format!("{}://127.0.0.1:{}", prefix, port);
                    let ipv6_web3 = Web3::new(&ipv6_url, timeout);
                    let ipv4_web3 = Web3::new(&ipv4_url, timeout);
                    let ipv6_test = ipv6_web3.eth_block_number().await;
                    let ipv4_test = ipv4_web3.eth_block_number().await;
                    warn!("Trying fallback urls {} {}", ipv6_url, ipv4_url);
                    match (ipv4_test, ipv6_test) {
                        (Ok(_), Err(_)) => {
                            info!("Url fallback succeeded, your Ethereum rpc url {} has been corrected to {}", eth_rpc_url, ipv4_url);
                            web3 = Some(ipv4_web3)
                        }
                        (Err(_), Ok(_)) => {
                            info!("Url fallback succeeded, your Ethereum  rpc url {} has been corrected to {}", eth_rpc_url, ipv6_url);
                            web3 = Some(ipv6_web3)
                        },
                        (Ok(_), Ok(_)) => panic!("This should never happen? Why didn't things work the first time?"),
                        (Err(_), Err(_)) => panic!("Could not connect to Ethereum rpc, are you sure it's running and on the specified port? {}", eth_rpc_url)
                    }
                } else if url.port().is_none() || url.scheme() == "http" {
                    let body = url.host_str().unwrap_or_else(|| {
                        panic!("Ethereum rpc url contains no host? {}", eth_rpc_url)
                    });
                    // transparently upgrade to https if available, we can't transparently downgrade for obvious security reasons
                    let https_on_80_url = format!("https://{}:80", body);
                    let https_on_443_url = format!("https://{}:443", body);
                    let https_on_80_web3 = Web3::new(&https_on_80_url, timeout);
                    let https_on_443_web3 = Web3::new(&https_on_443_url, timeout);
                    let https_on_80_test = https_on_80_web3.eth_block_number().await;
                    let https_on_443_test = https_on_443_web3.eth_block_number().await;
                    warn!(
                        "Trying fallback urls {} {}",
                        https_on_443_url, https_on_80_url
                    );
                    match (https_on_80_test, https_on_443_test) {
                        (Ok(_), Err(_)) => {
                            info!("Https upgrade succeeded, your Ethereum rpc url {} has been corrected to {}", eth_rpc_url, https_on_80_url);
                            web3 = Some(https_on_80_web3)
                        },
                        (Err(_), Ok(_)) => {
                            info!("Https upgrade succeeded, your Ethereum rpc url {} has been corrected to {}", eth_rpc_url, https_on_443_url);
                            web3 = Some(https_on_443_web3)
                        },
                        (Ok(_), Ok(_)) => panic!("This should never happen? Why didn't things work the first time?"),
                        (Err(_), Err(_)) => panic!("Could not connect to Ethereum rpc, are you sure it's running and on the specified port? {}", eth_rpc_url)
                    }
                } else {
                    panic!("Could not connect to Ethereum rpc! please check your grpc url {} for errors {:?}", eth_rpc_url, e)
                }
            }
        }
    }

    Connections {
        web3,
        grpc,
        contact,
    }
}

/// Verify that a url has an http or https prefix
fn check_scheme(input: &Url, original_string: &str) {
    if !(input.scheme() == "http" || input.scheme() == "https") {
        panic!(
            "Your url {} has an invalid scheme, please chose http or https",
            original_string
        )
    }
}

/// This function will wait until the Cosmos node is ready, this is intended
/// for situations such as when a node is syncing or when a node is waiting on
/// a halted chain.
pub async fn wait_for_cosmos_node_ready(contact: &Contact) {
    const WAIT_TIME: Duration = Duration::from_secs(10);
    loop {
        let res = contact.get_syncing_status().await;
        match res {
            Ok(val) => {
                if !val.syncing {
                    break;
                } else {
                    info!("Cosmos node is syncing or waiting for the chain to start. Standing by")
                }
            }
            Err(e) => warn!(
                "Could not get syncing status, is your Cosmos node up? {:?}",
                e
            ),
        }
        delay_for(WAIT_TIME).await;
    }
}

/// This function checks the orchestrator delegate addresses
/// for consistency what this means is that it takes the Ethereum
/// address and Orchestrator address from the Orchestrator and checks
/// that both are registered and internally consistent.
pub async fn check_delegate_addresses(
    client: &mut PeggyQueryClient<Channel>,
    delegate_eth_address: EthAddress,
    delegate_orchestrator_address: CosmosAddress,
) {
    let eth_response = client
        .get_delegate_key_by_eth(QueryDelegateKeysByEthAddress {
            eth_address: delegate_eth_address.to_string(),
        })
        .await;
    let orchestrator_response = client
        .get_delegate_key_by_orchestrator(QueryDelegateKeysByOrchestratorAddress {
            orchestrator_address: delegate_orchestrator_address.to_string(),
        })
        .await;
    trace!("{:?} {:?}", eth_response, orchestrator_response);
    match (eth_response, orchestrator_response) {
        (Ok(e), Ok(o)) => {
            let e = e.into_inner();
            let o = o.into_inner();
            let req_delegate_orchestrator_address: CosmosAddress =
                e.orchestrator_address.parse().unwrap();
            let req_delegate_eth_address: EthAddress = o.eth_address.parse().unwrap();
            if req_delegate_eth_address != delegate_eth_address
                && req_delegate_orchestrator_address != delegate_orchestrator_address
            {
                error!("Your Delegate Ethereum and Orchestrator addresses are both incorrect!");
                error!(
                    "You provided {}  Correct Value {}",
                    delegate_eth_address, req_delegate_eth_address
                );
                error!(
                    "You provided {}  Correct Value {}",
                    delegate_orchestrator_address, req_delegate_orchestrator_address
                );
                error!("In order to resolve this issue you should double check your input value or re-register your delegate keys");
                exit(1);
            } else if req_delegate_eth_address != delegate_eth_address {
                error!("Your Delegate Ethereum address is incorrect!");
                error!(
                    "You provided {}  Correct Value {}",
                    delegate_eth_address, req_delegate_eth_address
                );
                error!("In order to resolve this issue you should double check how you input your eth private key");
                exit(1);
            } else if req_delegate_orchestrator_address != delegate_orchestrator_address {
                error!("Your Delegate Orchestrator address is incorrect!");
                error!(
                    "You provided {}  Correct Value {}",
                    delegate_eth_address, req_delegate_eth_address
                );
                error!("In order to resolve this issue you should double check how you input your Orchestrator address phrase, make sure you didn't use your Validator phrase!");
                exit(1);
            }

            if e.validator_address != o.validator_address {
                error!("You are using delegate keys from two different validator addresses!");
                error!("If you get this error message I would just blow everything away and start again");
                exit(1);
            }
        }
        (Err(_), Ok(_o)) => {
            error!("Your delegate Ethereum address is incorrect, please double check you private key. If you can't locate the correct private key register your delegate keys again and use the new value");
            exit(1);
        }
        (Ok(_e), Err(_)) => {
            error!("Your delegate Cosmos address is incorrect, please double check your phrase. If you can't locate the correct phrase register your delegate keys again and use the new value");
            exit(1);
        }
        (Err(_), Err(_)) => {
            error!("Delegate keys are not set! Please Register your delegate keys");
            exit(1);
        }
    }
}