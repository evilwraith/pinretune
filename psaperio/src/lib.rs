// Copyright 2026 Martin Åkesson
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing
// permissions and limitations under the License.

//! PS Aperio - A PinSound Utility Library
//! ======================================
//!
//! This library provides utility functions for PinSound sound packs and configuration
//! files.
//!
//! This library provides the following functionality:
//! - [x] Scrambling and unscrambling of sound command IDs.
//! - [x] Reading encrypted files
//! - [x] Writing encrypted files
//!
//! *This library is not affiliated, associated, authorized, endorsed by, or in any way
//! connected with PinSound, SARL, or any of its subsidiaries or its affiliates.
//! PinSound is a registered trademark of PinSound, SARL.*

mod bytes;
pub mod file;
pub mod scrambler;
pub mod util;
