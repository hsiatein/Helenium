pub mod health;
mod kernel;
pub mod service;
pub use kernel::*;

extern crate service_auth;
extern crate service_chat;
extern crate service_config;
extern crate service_fs;
extern crate service_hub;
extern crate service_memory;
extern crate service_stats;
extern crate service_task;
extern crate service_test;
extern crate service_toolkit;
extern crate service_user;
extern crate service_webui;

extern crate service_schedule;

extern crate service_docker;
extern crate service_mcp;
extern crate service_process;

extern crate service_tools;
extern crate service_embed;