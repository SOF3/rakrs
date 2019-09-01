// rakrs
// Copyright (C) SOFe
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use derive_more::*;

/// A wrapper over `u32`, with only the three least-significant bytes encoded.
#[derive(Clone, Copy, Debug, Default, From, Into, PartialEq, Eq, PartialOrd, Ord)]
pub struct Triad(u32); // TODO check overflow
