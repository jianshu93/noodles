# Changelog

## Unreleased

### Added

  * bcf/async/reader: Add conversion from `R` into `Reader<R>`.

### Changed

  * bcf/header/string_map: Parsing can now fail with
    `vcf::header::ParseError::StringMapPositionMismatch` if the string map
    position of an entry and record-defined IDX field value do not match.

  * bcf/header/string_map: If present, the IDX field is used to determine the
    position of the entry in the string map ([#64]).

[#64]: https://github.com/zaeleus/noodles/issues/64

### Removed

  * bcf/header/string_map: Remove `Deref<Target = IndexSet<String>>` for
    `StringMap`.

    `StringMap` is no longer backed by an `IndexMap`.

## 0.10.0 - 2021-12-16

### Added

  * bcf/reader: Add common methods to access the underlying reader: `get_ref`,
    `get_mut` and `into_inner`.

  * bcf/writer: Add record writer (`Writer::write_record`).

### Fixed

  * bcf/writer/vcf_record/site: Fix allele count calculation.

    This overcounted by 1 when there were no alternate bases.

## 0.9.0 - 2021-12-02

### Added

  * bcf/record/genotypes: Add method to clear all fields (`Genotypes::clear`).

  * bcf/record/info: Add methods to wrap an INFO buffer (`Info::new`), clear
    all fields (`Info::clear`), retrieve a field by key (`Info::get`; [#52]),
    and iterate all fields (`Info::values`).

  * bcf/reader: Add conversion from `R` into `Reader<R>`.

  * bcf/writer: Add conversion from `W` into `Writer<W>`.

  * bcf/writer: Add common methods to access the underlying writer: `get_mut`
    and `into_inner`.

[#52]: https://github.com/zaeleus/noodles/issues/52

### Changed

  * bcf/header/string_map: Disable type checking when the file format is < VCF
    4.3.

  * bcf/reader/record/genotypes: Always use key from header.

  * bcf/reader/record/site/info: Always use key from header.

  * bcf/record/genotypes: Return `Genotypes` from
    `Genotypes::try_into_vcf_record_genotypes`.

    Use `vcf::record::Genotypes::keys` for the genotypes keys.

### Fixed

  * bcf/reader/record/site/info: Allow value to be optional (#56).

[#56]: https://github.com/zaeleus/noodles/issues/56

## 0.8.0 - 2021-11-18

### Added

  * bcf/record: Implement `Debug` for `Record`.

  * bcf/record: Add getter for filters (`Record::filters`), IDs
    (`Record::ids`), genotypes (`Record::genotypes`), info (`Record::info`),
    and quality score (`Record::quality_score`).

  * bcf/record: Add mutable getters for IDs (`Record::ids_mut`), position
    (`Record::position_mut`) and quality score (`Record::quality_score_mut`).

### Changed

  * bcf/record: `bcf::Record` is no longer backed by a contiguous buffer.

    Fields are read individually when reading the record. `bcf::Record` no
    longer implements `Deref<Target = [u8]>`. `Filters`, `Info`, `Genotypes`
    now own their data.

## 0.7.0 - 2021-11-11

### Changed

  * bcf: Update to Rust 2021.

## 0.6.1 - 2021-10-16

### Fixed

  * bcf: Sync dependencies.

## 0.6.0 - 2021-10-01

### Added

  * bcf: Increase visibility of `reader` module ([#37]).

    This allows public access to the reader iterators `Records` and `Query`.

[#37]: https://github.com/zaeleus/noodles/pull/37

## 0.5.2 - 2021-09-19

### Fixed

  * bcf: Sync dependencies.

## 0.5.1 - 2021-09-01

### Fixed

  * bcf: Sync dependencies.

## 0.5.0 - 2021-08-19

### Changed

  * bcf: Update to tokio 1.10.0.

  * bcf/async: I/O builders are now owned/consuming builders.

    This fixes the terminal method not being able to move out of a mutable
    reference.

### Fixed

  * bcf: Define features to enable for Docs.rs.

## 0.4.0 - 2021-08-11

### Added

  * bcf/async: Add async reader (`bcf::AsyncReader`).

    This can be enabled with the `async` feature.

## 0.3.1 - 2021-08-04

### Fixed

  * bcf: Sync dependencies.

## 0.3.0 - 2021-07-30

### Added

  * bcf/header/string_map: Implement conversion from `vcf::Header`.

## 0.2.0 - 2021-07-21

### Added

  * bcf/reader: Accept any `BinningIndex` to query.

### Fixed

  * bcf: Fixed documentation link in package manifest ([#31]).

[#31]: https://github.com/zaeleus/noodles/issues/31

## 0.1.0 - 2021-07-14

  * bcf: Initial release.
