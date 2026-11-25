// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! What the loop's parts share.

use core::time::Duration;

/// Cap on the back-off, and the wait after "unreachable".
pub const MAX_BACKOFF: Duration = Duration::from_hours(6);
