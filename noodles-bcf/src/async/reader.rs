mod builder;

pub use self::builder::Builder;

use futures::{stream, Stream};
use noodles_bgzf as bgzf;
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncSeek};

use crate::Record;

/// An async BCF reader.
///
/// # Examples
///
/// ```no_run
/// # use std::io;
/// #
/// # #[tokio::main]
/// # async fn main() -> io::Result<()> {
/// use futures::TryStreamExt;
/// use noodles_bcf as bcf;
/// use tokio::fs::File;
///
/// let mut reader = File::open("sample.bcf").await.map(bcf::AsyncReader::new)?;
/// reader.read_file_format().await?;
/// reader.read_header().await?;
///
/// let mut records = reader.records();
///
/// while let Some(record) = records.try_next().await? {
///     // ...
/// }
/// # Ok(())
/// # }
/// ```
pub struct Reader<R> {
    inner: R,
    buf: Vec<u8>,
}

impl<R> Reader<R>
where
    R: AsyncRead + Unpin,
{
    /// Reads the BCF file format.
    ///
    /// The BCF magic number is also checked.
    ///
    /// The position of the stream is expected to be at the start.
    ///
    /// This returns the major and minor format versions as a tuple.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> io::Result<()> {
    /// use noodles_bcf as bcf;
    /// use tokio::fs::File;
    /// let mut reader = File::open("sample.bcf").await.map(bcf::AsyncReader::new)?;
    /// let (major, minor) = reader.read_file_format().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_file_format(&mut self) -> io::Result<(u8, u8)> {
        read_magic(&mut self.inner).await?;
        read_format_version(&mut self.inner).await
    }

    /// Reads the raw VCF header.
    ///
    /// The position of the stream is expected to be directly after the file format.
    ///
    /// This returns the raw VCF header as a [`String`]. It can subsequently be parsed as a
    /// [`noodles_vcf::Header`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> io::Result<()> {
    /// use noodles_bcf as bcf;
    /// use tokio::fs::File;
    ///
    /// let mut reader = File::open("sample.bcf").await.map(bcf::AsyncReader::new)?;
    /// reader.read_file_format().await?;
    ///
    /// let header = reader.read_header().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_header(&mut self) -> io::Result<String> {
        read_header(&mut self.inner).await
    }

    /// Reads a single record.
    ///
    /// The stream is expected to be directly after the header or at the start of another record.
    ///
    /// It is more ergonomic to read records using a stream (see [`Self::records`]), but using this
    /// method directly allows the reuse of a single [`Record`] buffer.
    ///
    /// If successful, the record size is returned. If a record size of 0 is returned, the stream
    /// reached EOF.
    ///
    /// ```no_run
    /// # use std::io;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> io::Result<()> {
    /// use noodles_bcf as bcf;
    /// use tokio::fs::File;
    ///
    /// let mut reader = File::open("sample.bcf").await.map(bcf::AsyncReader::new)?;
    /// reader.read_file_format().await?;
    /// reader.read_header().await?;
    ///
    /// let mut record = bcf::Record::default();
    /// reader.read_record(&mut record).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_record(&mut self, record: &mut Record) -> io::Result<usize> {
        read_record(&mut self.inner, &mut self.buf, record).await
    }

    /// Returns an (async) stream over records starting from the current (input) stream position.
    ///
    /// The (input) stream is expected to be directly after the header or at the start of another
    /// record.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> io::Result<()> {
    /// use futures::TryStreamExt;
    /// use noodles_bcf as bcf;
    /// use tokio::fs::File;
    ///
    /// let mut reader = File::open("sample.bcf").await.map(bcf::AsyncReader::new)?;
    /// reader.read_file_format().await?;
    /// reader.read_header().await?;
    ///
    /// let mut records = reader.records();
    ///
    /// while let Some(record) = records.try_next().await? {
    ///     // ...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn records(&mut self) -> impl Stream<Item = io::Result<Record>> + '_ {
        Box::pin(stream::try_unfold(
            (&mut self.inner, Vec::new(), Record::default()),
            |(mut reader, mut buf, mut record)| async {
                read_record(&mut reader, &mut buf, &mut record)
                    .await
                    .map(|n| match n {
                        0 => None,
                        _ => Some((record.clone(), (reader, buf, record))),
                    })
            },
        ))
    }
}

impl<R> Reader<bgzf::AsyncReader<R>>
where
    R: AsyncRead + Unpin,
{
    /// Creates an async BCF reader builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf as bcf;
    /// let data = [];
    /// let builder = bcf::AsyncReader::builder(&data[..]);
    /// let reader = builder.build();
    /// ```
    pub fn builder(inner: R) -> Builder<R> {
        Builder::new(inner)
    }

    /// Creates an async BCF reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf as bcf;
    /// let data = [];
    /// let reader = bcf::AsyncReader::new(&data[..]);
    /// ```
    pub fn new(inner: R) -> Self {
        Self::from(bgzf::AsyncReader::new(inner))
    }

    /// Returns the current virtual position of the underlying BGZF reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bcf as bcf;
    /// use noodles_bgzf as bgzf;
    ///
    /// let data = Vec::new();
    /// let reader = bcf::AsyncReader::new(&data[..]);
    /// let virtual_position = reader.virtual_position();
    ///
    /// assert_eq!(reader.virtual_position(), bgzf::VirtualPosition::from(0));
    /// ```
    pub fn virtual_position(&self) -> bgzf::VirtualPosition {
        self.inner.virtual_position()
    }
}

impl<R> Reader<bgzf::AsyncReader<R>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    /// Seeks the underlying BGZF reader to the given virtual position.
    ///
    /// Virtual positions typically come from an associated index file.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::{self, Cursor};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> io::Result<()> {
    /// use noodles_bcf as bcf;
    /// use noodles_bgzf as bgzf;
    ///
    /// let data = [];
    /// let mut reader = bcf::AsyncReader::new(Cursor::new(data));
    ///
    /// let virtual_position = bgzf::VirtualPosition::default();
    /// reader.seek(virtual_position).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn seek(&mut self, pos: bgzf::VirtualPosition) -> io::Result<bgzf::VirtualPosition> {
        self.inner.seek(pos).await
    }
}

impl<R> From<R> for Reader<R> {
    fn from(inner: R) -> Self {
        Self {
            inner,
            buf: Vec::new(),
        }
    }
}

async fn read_magic<R>(reader: &mut R) -> io::Result<()>
where
    R: AsyncRead + Unpin,
{
    use crate::MAGIC_NUMBER;

    let mut magic = [0; 3];
    reader.read_exact(&mut magic).await?;

    if magic == MAGIC_NUMBER {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid BCF header",
        ))
    }
}

async fn read_format_version<R>(reader: &mut R) -> io::Result<(u8, u8)>
where
    R: AsyncRead + Unpin,
{
    let major_version = reader.read_u8().await?;
    let minor_version = reader.read_u8().await?;

    Ok((major_version, minor_version))
}

async fn read_header<R>(reader: &mut R) -> io::Result<String>
where
    R: AsyncRead + Unpin,
{
    let l_text = reader.read_u32_le().await.and_then(|len| {
        usize::try_from(len).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    })?;

    let mut buf = vec![0; l_text];
    reader.read_exact(&mut buf).await?;

    c_str_to_string(&buf)
}

async fn read_record<R>(reader: &mut R, buf: &mut Vec<u8>, record: &mut Record) -> io::Result<usize>
where
    R: AsyncRead + Unpin,
{
    use crate::reader::record::read_site;

    let l_shared = match reader.read_u32_le().await {
        Ok(n) => usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(0),
        Err(e) => return Err(e),
    };

    let l_indiv = reader.read_u32_le().await.and_then(|n| {
        usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    })?;

    buf.resize(l_shared, Default::default());
    reader.read_exact(buf).await?;
    let mut reader = &buf[..];
    let (n_fmt, n_sample) = read_site(&mut reader, record)?;

    let genotypes = record.genotypes_mut().as_mut();
    genotypes.resize(l_indiv, Default::default());
    reader.read_exact(genotypes).await?;
    record.genotypes_mut().set_format_count(n_fmt);
    record.genotypes_mut().set_sample_count(n_sample);

    Ok(l_shared + l_indiv)
}

fn c_str_to_string(buf: &[u8]) -> io::Result<String> {
    use std::ffi::CStr;

    CStr::from_bytes_with_nul(buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        .and_then(|c_header| {
            c_header
                .to_str()
                .map(|s| s.into())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_magic() {
        let data = b"BCF";
        let mut reader = &data[..];
        assert!(read_magic(&mut reader).await.is_ok());

        let data = [];
        let mut reader = &data[..];
        assert!(matches!(
            read_magic(&mut reader).await,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof
        ));

        let data = b"BAM";
        let mut reader = &data[..];
        assert!(matches!(
            read_magic(&mut reader).await,
            Err(ref e) if e.kind() == io::ErrorKind::InvalidData
        ));
    }

    #[tokio::test]
    async fn test_read_format_version() -> io::Result<()> {
        let data = [0x02, 0x01];
        let mut reader = &data[..];
        assert_eq!(read_format_version(&mut reader).await?, (2, 1));
        Ok(())
    }

    #[tokio::test]
    async fn test_read_header() -> io::Result<()> {
        let data = [
            0x08, 0x00, 0x00, 0x00, // l_text = 8
            0x6e, 0x6f, 0x6f, 0x64, 0x6c, 0x65, 0x73, 0x00, // text = b"noodles\x00"
        ];

        let mut reader = &data[..];
        assert_eq!(read_header(&mut reader).await?, "noodles");

        Ok(())
    }
}
