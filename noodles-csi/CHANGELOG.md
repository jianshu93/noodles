# Changelog

## 0.4.2 - 2021-12-02

### Fixed

  * csi: Require tokio's `fs` feature as a dependency ([#62]).

[#62]: https://github.com/zaeleus/noodles/issues/62

## 0.4.1 - 2021-11-18

### Fixed

  * csi: Sync dependencies.

## 0.4.0 - 2021-11-11

### Changed

  * csi: Update to Rust 2021.

### Deprecated

  * csi/binning_index: Rename `csi::BinningIndexReferenceSequence` to
    `csi::binning_index::ReferenceSequenceExt`.

## 0.3.0 - 2021-08-19

### Added

  * csi/async: Add async reader (`csi::AsyncReader`).

  * csi/async: Add async writer (`csi::AsyncWriter`).

    Async I/O can be enabled with the `async` feature.

## 0.2.2 - 2021-08-11

### Fixed

  * csi: Sync dependencies.

## 0.2.1 - 2021-07-30

### Fixed

  * csi/reader: Return I/O errors when failing to read `n_no_coor`.

    This previously ignored all I/O errors but now only catches
    `UnexpectedEof`.

## 0.2.0 - 2021-07-21

### Added

  * csi: Add convenience function to write an entire index to a file:
    `csi::write`.

  * csi/binning_index: Added chunk merging functions for chunk list reduction
    (`noodles_csi::binning_index::{merge_chunks, optimize_chunks}`).

    Chunks are merged when they overlap and can be filtered by a minimum
    offset.

  * csi/binning_index: Added `BinningIndex` and `BinningIndexReferenceSequence`
    traits to define shared behavior among binning index formats.

  * csi/binning_index: Added `first_record_in_last_linear_bin_start_position`.

    This is the closest position to the unplaced, unmapped records, if any,
    that is available in an index.

  * csi/index: Implemented `BinningIndex` for `Index`.

  * csi/index: Added `query` method to find chunks that intersect the given
    region.

  * csi/index/reference_sequence: Implemented `BinningIndexReferenceSequence`
    for `ReferenceSequence`.

### Deprecated

  * csi/index: Deprecated `Index::unmapped_read_count`.

    Use `unplaced_unmapped_record_count` instead.

  * csi/index/builder: Deprecated `Builder::set_n_no_coor`.

    Use `set_unplaced_unmapped_record_count` instead.

### Fixed

  * csi: Fixed documentation link in package manifest ([#31]).

[#31]: https://github.com/zaeleus/noodles/issues/31

## 0.1.0 - 2021-07-14

  * csi: Initial release.
