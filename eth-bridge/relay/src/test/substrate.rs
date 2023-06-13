// This file is part of Substrate.

// Copyright (C) 2020-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

// File has been modified by Liberland in 2023. All modifications by Liberland are distributed under
// the MIT license.

// You should have received a copy of the MIT license along with this program. If not, see https://opensource.org/licenses/MIT

#![cfg(unix)]

use std::ops::{Deref, DerefMut};
use tokio::{
	io::{AsyncBufReadExt, AsyncRead, BufReader},
	process::Child,
};

pub struct KillChildOnDrop(pub Child);

impl Drop for KillChildOnDrop {
	fn drop(&mut self) {
		self.0.start_kill().ok();
	}
}

impl Deref for KillChildOnDrop {
	type Target = Child;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for KillChildOnDrop {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Read the WS address from the output.
///
/// This is hack to get the actual binded sockaddr because
/// substrate assigns a random port if the specified port was already binded.
pub async fn find_ws_url_from_output(read: impl AsyncRead + Send + Unpin) -> String {
	let mut data = String::new();

	let mut lines = BufReader::new(read).lines();
	while let Some(line) = lines.next_line().await.unwrap() {
		data.push_str(&line);
		data.push_str("\n");

		// does the line contain our port (we expect this specific output from substrate).
		let sock_addr = match line.split_once("Running JSON-RPC WS server: addr=") {
			None => continue,
			Some((_, after)) => after.split_once(",").unwrap().0,
		};

		return format!("ws://{}", sock_addr)
	}

	eprintln!("Observed node output:\n{}", data);
	panic!("We should get a WebSocket address");
}
