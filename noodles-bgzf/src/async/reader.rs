mod block_decoder;
mod blocks;

use std::{
    cmp, io,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{stream::TryBuffered, Stream, StreamExt, TryStreamExt};
use pin_project_lite::pin_project;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncSeek, ReadBuf};

use crate::{Block, VirtualPosition};

use self::blocks::Blocks;

const WORKER_COUNT: usize = 8;

pin_project! {
    /// An async BGZF reader.
    pub struct Reader<R>
    where
        R: AsyncRead,
    {
        #[pin]
        stream: Option<TryBuffered<Blocks<R>>>,
        block: Block,
    }
}

impl<R> Reader<R>
where
    R: AsyncRead,
{
    /// Creates an async BGZF reader.
    pub fn new(inner: R) -> Self {
        Self {
            stream: Some(Blocks::new(inner).try_buffered(WORKER_COUNT)),
            block: Block::default(),
        }
    }

    /// Returns the current virtual position of the stream.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_bgzf as bgzf;
    /// let data = [];
    /// let reader = bgzf::AsyncReader::new(&data[..]);
    /// assert_eq!(reader.virtual_position(), bgzf::VirtualPosition::from(0));
    /// # Ok::<(), io::Error>(())
    /// ```
    pub fn virtual_position(&self) -> VirtualPosition {
        self.block.virtual_position()
    }
}

impl<R> Reader<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    /// Seeks the stream to the given virtual position.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::{self, Cursor};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> io::Result<()> {
    /// use noodles_bgzf as bgzf;
    /// let mut reader = bgzf::AsyncReader::new(Cursor::new(Vec::new()));
    /// let virtual_position = bgzf::VirtualPosition::from(102334155);
    /// reader.seek(virtual_position).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn seek(&mut self, pos: VirtualPosition) -> io::Result<VirtualPosition> {
        let stream = self.stream.take().expect("missing stream");
        let mut blocks = stream.into_inner();

        blocks.seek(pos).await?;

        let mut stream = blocks.try_buffered(WORKER_COUNT);

        self.block = match stream.next().await {
            Some(Ok(mut block)) => {
                let (cpos, upos) = pos.into();
                block.set_cpos(cpos);
                block.set_upos(u32::from(upos));
                block
            }
            Some(Err(e)) => return Err(e),
            None => Block::default(),
        };

        self.stream.replace(stream);

        Ok(pos)
    }
}

impl<R> AsyncRead for Reader<R>
where
    R: AsyncRead,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let block_buf = match self.as_mut().poll_fill_buf(cx) {
            Poll::Ready(Ok(b)) => b,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        };

        let len = cmp::min(block_buf.len(), buf.remaining());
        buf.put_slice(&block_buf[..len]);

        self.consume(len);

        Poll::Ready(Ok(()))
    }
}

impl<R> AsyncBufRead for Reader<R>
where
    R: AsyncRead,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        let this = self.project();

        if this.block.is_eof() {
            let stream = this.stream.as_pin_mut().expect("missing stream");

            match stream.poll_next(cx) {
                Poll::Ready(Some(Ok(block))) => {
                    *this.block = block;
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Err(e)),
                Poll::Ready(None) => return Poll::Ready(Ok(&[])),
                Poll::Pending => return Poll::Pending,
            }
        }

        return Poll::Ready(Ok(this.block.fill_buf()));
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let this = self.project();
        let upos = cmp::min(this.block.ulen(), this.block.upos() + amt as u32);
        this.block.set_upos(upos);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tokio::io::AsyncReadExt;

    use crate::Writer;

    use super::*;

    #[tokio::test]
    async fn test_read() -> io::Result<()> {
        let mut writer = Writer::new(Vec::new());
        writer.write_all(b"noodles")?;

        let data = writer.finish()?;
        let mut reader = Reader::new(&data[..]);

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        assert_eq!(buf, b"noodles");

        Ok(())
    }
}