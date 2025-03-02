use std::{
    io::{self, Read, Seek},
    ops::{Bound, RangeBounds},
};

use noodles_bgzf as bgzf;
use noodles_csi::index::reference_sequence::bin::Chunk;

use super::Reader;
use crate::{record::Chromosome, Header, Record};

enum State {
    Seek,
    Read(bgzf::VirtualPosition),
    End,
}

/// An iterator over records of a VCF reader that intersects a given region.
///
/// This is created by calling [`Reader::query`].
pub struct Query<'r, 'h, R>
where
    R: Read + Seek + 'r,
{
    reader: &'r mut Reader<bgzf::Reader<R>>,
    chunks: Vec<Chunk>,
    reference_sequence_name: String,
    start: i32,
    end: i32,
    i: usize,
    state: State,
    header: &'h Header,
    line_buf: String,
}

impl<'r, 'h, R> Query<'r, 'h, R>
where
    R: Read + Seek,
{
    pub(crate) fn new<B>(
        reader: &'r mut Reader<bgzf::Reader<R>>,
        chunks: Vec<Chunk>,
        reference_sequence_name: String,
        interval: B,
        header: &'h Header,
    ) -> Self
    where
        B: RangeBounds<i32>,
    {
        let (start, end) = match (interval.start_bound(), interval.end_bound()) {
            (Bound::Unbounded, Bound::Unbounded) => (1, i32::MAX),
            (Bound::Included(s), Bound::Unbounded) => (*s, i32::MAX),
            (Bound::Included(s), Bound::Included(e)) => (*s, *e),
            _ => todo!(),
        };

        Self {
            reader,
            chunks,
            reference_sequence_name,
            start,
            end,
            i: 0,
            state: State::Seek,
            header,
            line_buf: String::new(),
        }
    }

    fn next_chunk(&mut self) -> io::Result<Option<bgzf::VirtualPosition>> {
        if self.i >= self.chunks.len() {
            return Ok(None);
        }

        let chunk = self.chunks[self.i];
        self.reader.seek(chunk.start())?;

        self.i += 1;

        Ok(Some(chunk.end()))
    }

    fn read_and_parse_record(&mut self) -> Option<io::Result<Record>> {
        self.line_buf.clear();

        match self.reader.read_record(&mut self.line_buf) {
            Ok(0) => None,
            Ok(_) => Some(
                Record::try_from_str(&self.line_buf, self.header)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            ),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'r, 'h, R> Iterator for Query<'r, 'h, R>
where
    R: Read + Seek,
{
    type Item = io::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                State::Seek => {
                    self.state = match self.next_chunk() {
                        Ok(Some(chunk_end)) => State::Read(chunk_end),
                        Ok(None) => State::End,
                        Err(e) => return Some(Err(e)),
                    }
                }
                State::Read(chunk_end) => match self.read_and_parse_record() {
                    Some(result) => {
                        if self.reader.virtual_position() >= chunk_end {
                            self.state = State::Seek;
                        }

                        match result {
                            Ok(record) => {
                                let reference_sequence_name = match record.chromosome() {
                                    Chromosome::Name(n) => n.into(),
                                    Chromosome::Symbol(n) => n.to_string(),
                                };

                                let start = i32::from(record.position());

                                let end = match record.end() {
                                    Ok(pos) => i32::from(pos),
                                    Err(e) => {
                                        return Some(Err(io::Error::new(
                                            io::ErrorKind::InvalidData,
                                            e,
                                        )))
                                    }
                                };

                                if reference_sequence_name == self.reference_sequence_name
                                    && in_interval(start, end, self.start, self.end)
                                {
                                    return Some(Ok(record));
                                }
                            }
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    None => {
                        self.state = State::Seek;
                    }
                },
                State::End => return None,
            }
        }
    }
}

fn in_interval(a_start: i32, a_end: i32, b_start: i32, b_end: i32) -> bool {
    a_start <= b_end && b_start <= a_end
}
