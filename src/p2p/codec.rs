use tokio::io;
use tokio::net::{TcpStream};
use tokio::prelude::*;
use bytes::{BytesMut, BufMut};


/// Line based codec
///
/// This decorates a socket and presents a line based read / write interface.
///
/// As a user of `Lines`, we can focus on working at the line level. So, we send
/// and receive values that represent entire lines. The `Lines` codec will
/// handle the encoding and decoding as well as reading from and writing to the
/// socket.
#[derive(Debug)]
pub struct Lines {
    /// The TCP socket.
    pub socket: TcpStream,

    /// Buffer used when reading from the socket. Data is not returned from this
    /// buffer until an entire line has been read.
    rd: BytesMut,

    /// Buffer used to stage data before writing it to the socket.
    wr: BytesMut,
}

impl Lines {
    /// Create a new `Lines` codec backed by the socket
    pub fn new(socket: TcpStream) -> Self {
        Lines {
            socket,
            rd: BytesMut::new(),
            wr: BytesMut::new(),
        }
    }

    /// Buffer a line.
    ///
    /// This writes the line to an internal buffer. Calls to `poll_flush` will
    /// attempt to flush this buffer to the socket.
    pub fn buffer(&mut self, line: &[u8]) {
        // Ensure the buffer has capacity. Ideally this would not be unbounded,
        // but to keep the example simple, we will not limit this.
        self.wr.reserve(line.len());

        // Push the line onto the end of the write buffer.
        //
        // The `put` function is from the `BufMut` trait.
        self.wr.put(line);
    }

    /// Flush the write buffer to the socket
    pub fn poll_flush(&mut self) -> Poll<(), io::Error> {
        // As long as there is buffered data to write, try to write it.
        while !self.wr.is_empty() {
            // Try to write some bytes to the socket
            let n = try_ready!(self.socket.poll_write(&self.wr));

            // As long as the wr is not empty, a successful write should
            // never write 0 bytes.
            assert!(n > 0);

            // This discards the first `n` bytes of the buffer.
            let _ = self.wr.split_to(n);
        }

        Ok(Async::Ready(()))
    }

    /// Read data from the socket.
    ///
    /// This only returns `Ready` when the socket has closed.
    fn fill_read_buf(&mut self) -> Poll<(), io::Error> {
        loop {
            // Ensure the read buffer has capacity.
            //
            // This might result in an internal allocation.
            self.rd.reserve(1024);

            // Read data into the buffer.
            let n = try_ready!(self.socket.read_buf(&mut self.rd));

            if n == 0 {
                return Ok(Async::Ready(()));
            }
        }
    }
}

impl Stream for Lines {
    type Item = BytesMut;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // First, read any new data that might have been received off the socket
        let sock_closed = self.fill_read_buf()?.is_ready();

        // Now, try finding lines
        let pos = self.rd.windows(2).enumerate()
            .find(|&(_, bytes)| bytes == b"\r\n")
            .map(|(i, _)| i);

        if let Some(pos) = pos {
            // Remove the line from the read buffer and set it to `line`.
            let mut line = self.rd.split_to(pos + 2);

            // Drop the trailing \r\n
            line.split_off(pos);

            // Return the line
            return Ok(Async::Ready(Some(line)));
        }

        if sock_closed {
            Ok(Async::Ready(None))
        } else {
            Ok(Async::NotReady)
        }
    }
}