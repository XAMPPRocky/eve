//! # Eve: Environment editor
//! The **eve** utility reads the specified files, or standard input if no files
//! are specified, replacing all instances of `{{VAR}}` with the environment
//! variable of the same name e.g. `$VAR`. This utility is mainly useful as a
//! replacement to using `sed` to insert environment variables into files. As is
//! common when using Docker.
//!
//! ## Example
//! Here's an example of replacing variables in a nginx configuration with
//! environment variables, and comparsion with the equivalent `sed` command.
//!
//! #### `nginx.conf`
//! ```nginx
//! server {
//!     listen 80;
//!     listen [::]:80;
//!
//!     server_name {{NGINX_HOST}};
//!
//!     location / {
//!         proxy_pass {{NGINX_PROXY}};
//!         proxy_next_upstream error timeout invalid_header http_500 http_502
//!             http_503 http_504;
//!         proxy_redirect off;
//!         proxy_buffering off;
//!         proxy_set_header        Host            {{NGINX_HOST}};
//!         proxy_set_header        X-Real-IP       $remote_addr;
//!         proxy_set_header        X-Forwarded-For $proxy_add_x_forwarded_for;
//!     }
//! }
//! ```
//!
//! #### `.env`
//! ```text
//! NGINX_HOST=localhost
//! NGINX_PROXY=localhost:8000
//! ```
//!
//! #### Commands
//!
//! ###### `sed`
//! ```bash
//! sed -e "s|{{NGINX_HOST}}|$NGINX_HOST|" \
//!     -e "s|{{NGINX_PROXY}}|$NGINX_PROXY|" \
//!     nginx.conf
//! ```
//! ###### `eve`
//! ```bash
//! eve nginx.conf
//! ```
//!
//! #### Output
//! ```bash
//! server {
//!     listen 80;
//!     listen [::]:80;
//!
//!     server_name localhost;
//!
//!     location / {
//!         proxy_pass localhost:8000;
//!         proxy_next_upstream error timeout invalid_header http_500 http_502
//!             http_503 http_504;
//!         proxy_redirect off;
//!         proxy_buffering off;
//!         proxy_set_header        Host            localhost;
//!         proxy_set_header        X-Real-IP       $remote_addr;
//!         proxy_set_header        X-Forwarded-For $proxy_add_x_forwarded_for;
//!     }
//! }
//! ```

extern crate regex;
extern crate dotenv;

use std::borrow::Cow;
use std::path::Path;
use std::env;

use regex::{Captures, Regex, Replacer};

/// A struct to allow replacement of text with environment variables.
#[derive(Clone, Copy, Debug)]
pub struct Eve;

impl Eve {
    /// Creates a new `Eve` using environment variables from `./.env`.
    pub fn new() -> Result<Self, dotenv::Error> {
        dotenv::dotenv()?;
        Ok(Eve)
    }

    /// Creates a new `Eve` using environment variables from the path specified
    /// by the `path` variables.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, dotenv::Error> {
        dotenv::from_path(path)?;
        Ok(Eve)
    }

    /// Perform a replacement on the provided `text`.
    pub fn replace<'e, 's>(&'e self, text: &'s str)
        -> Result<Cow<'s, str>, regex::Error>
    {
        let regex = Regex::new(r"\{\{(.*)\}\}")?;

        // `*self` used so that we can have `&self` without
        // requiring `&mut self`.
        Ok(regex.replace_all(text, *self))
    }
}

impl Replacer for Eve {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        let env_var = caps.get(1).expect("Expected 2 captures,\
                                         is there a capture group in the braces\
                                         ?");

        dst.extend(env::var(env_var.as_str()))
    }
}
