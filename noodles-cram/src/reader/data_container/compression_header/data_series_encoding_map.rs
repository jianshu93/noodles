use std::io::{self, Read};

use super::read_encoding;
use crate::{
    data_container::compression_header::{
        data_series_encoding_map::DataSeries, DataSeriesEncodingMap,
    },
    reader::num::read_itf8,
};

pub fn read_data_series_encoding_map<R>(reader: &mut R) -> io::Result<DataSeriesEncodingMap>
where
    R: Read,
{
    let data_len = read_itf8(reader).and_then(|n| {
        usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    })?;

    let mut buf = vec![0; data_len];
    reader.read_exact(&mut buf)?;

    let mut buf_reader = &buf[..];
    let map_len = read_itf8(&mut buf_reader)?;

    let mut builder = DataSeriesEncodingMap::builder();
    let mut key_buf = [0; 2];

    for _ in 0..map_len {
        buf_reader.read_exact(&mut key_buf)?;

        let key = DataSeries::try_from(key_buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let encoding = read_encoding(&mut buf_reader)?;

        builder = match key {
            DataSeries::BamBitFlags => builder.set_bam_bit_flags_encoding(encoding),
            DataSeries::CramBitFlags => builder.set_cram_bit_flags_encoding(encoding),
            DataSeries::ReferenceId => builder.set_reference_id_encoding(encoding),
            DataSeries::ReadLengths => builder.set_read_lengths_encoding(encoding),
            DataSeries::InSeqPositions => builder.set_in_seq_positions_encoding(encoding),
            DataSeries::ReadGroups => builder.set_read_groups_encoding(encoding),
            DataSeries::ReadNames => builder.set_read_names_encoding(encoding),
            DataSeries::NextMateBitFlags => builder.set_next_mate_bit_flags_encoding(encoding),
            DataSeries::NextFragmentReferenceSequenceId => {
                builder.set_next_fragment_reference_sequence_id_encoding(encoding)
            }
            DataSeries::NextMateAlignmentStart => {
                builder.set_next_mate_alignment_start_encoding(encoding)
            }
            DataSeries::TemplateSize => builder.set_template_size_encoding(encoding),
            DataSeries::DistanceToNextFragment => {
                builder.set_distance_to_next_fragment_encoding(encoding)
            }
            DataSeries::TagIds => builder.set_tag_ids_encoding(encoding),
            DataSeries::NumberOfReadFeatures => {
                builder.set_number_of_read_features_encoding(encoding)
            }
            DataSeries::ReadFeaturesCodes => builder.set_read_features_codes_encoding(encoding),
            DataSeries::InReadPositions => builder.set_in_read_positions_encoding(encoding),
            DataSeries::DeletionLengths => builder.set_deletion_lengths_encoding(encoding),
            DataSeries::StretchesOfBases => builder.set_stretches_of_bases_encoding(encoding),
            DataSeries::StretchesOfQualityScores => {
                builder.set_stretches_of_quality_scores_encoding(encoding)
            }
            DataSeries::BaseSubstitutionCodes => {
                builder.set_base_substitution_codes_encoding(encoding)
            }
            DataSeries::Insertion => builder.set_insertion_encoding(encoding),
            DataSeries::ReferenceSkipLength => builder.set_reference_skip_length_encoding(encoding),
            DataSeries::Padding => builder.set_padding_encoding(encoding),
            DataSeries::HardClip => builder.set_hard_clip_encoding(encoding),
            DataSeries::SoftClip => builder.set_soft_clip_encoding(encoding),
            DataSeries::MappingQualities => builder.set_mapping_qualities_encoding(encoding),
            DataSeries::Bases => builder.set_bases_encoding(encoding),
            DataSeries::QualityScores => builder.set_quality_scores_encoding(encoding),
            DataSeries::ReservedTc | DataSeries::ReservedTn => builder,
        }
    }

    builder
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_data(data_series_encoding_map: &DataSeriesEncodingMap) -> io::Result<Vec<u8>> {
        use crate::writer::data_container::compression_header::data_series_encoding_map::write_data_series_encoding_map;

        let mut buf = Vec::new();
        write_data_series_encoding_map(&mut buf, data_series_encoding_map)?;
        Ok(buf)
    }

    #[test]
    fn test_read_data_series_encoding_map() -> io::Result<()> {
        let expected = DataSeriesEncodingMap::default();

        let data = build_data(&expected)?;
        let mut reader = &data[..];
        let actual = read_data_series_encoding_map(&mut reader)?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
