# Changelog

All notable changes to this project will be documented in this file.
See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [0.1.3] - 2022-07-15

### Bug Fixes

- Initialize encoding tree with full size to avoid vec reallocation
- Prevent panic during word decoding with bad data

### Documentation

- Update timings

### Miscellaneous Tasks

- Cleanup: replace magic numbers by constants
- Cleanup test assets
- Set gitattributes for lorem_ipsum.txt for correct testing
- Update test and bench dependencies

## [0.1.2] - 2022-06-08

### Bug Fixes

- Fix BitReader read operation, would use read_exact instead of read
- Speed optimisation in the encoding tree.

### Documentation

- Calculate throughput in benchmarks
- Remove usage of `unwrap` in examples
- Rewrite bench, document speed

### Miscellaneous Tasks

- Mark satellite crates as "do not publish"
- Update gif cliff configuration

## [0.1.1] - 2022-06-07

### Documentation

- Fix README and documentation

## [0.1.0] - 2022-06-07

### Documentation

- Update examples

### Features

- Prepare for initial release

