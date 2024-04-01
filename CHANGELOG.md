# Changelog

## [0.34.0](https://github.com/momentohq/client-sdk-rust/compare/v0.33.1...v0.34.0) (2024-03-27)


### Features

* add ListCaches and initial control client integration tests ([#206](https://github.com/momentohq/client-sdk-rust/issues/206)) ([bec2651](https://github.com/momentohq/client-sdk-rust/commit/bec2651630905fd86a2091e2ccd0bc80a2d9d877))
* tweaks to builders ([#191](https://github.com/momentohq/client-sdk-rust/issues/191)) ([046d8ee](https://github.com/momentohq/client-sdk-rust/commit/046d8ee379694874d3d5baca000b2f31059e6ad0))
* use impl Into&lt;String&gt; on most public APIs ([#201](https://github.com/momentohq/client-sdk-rust/issues/201)) ([d5e136c](https://github.com/momentohq/client-sdk-rust/commit/d5e136c9071aedf80c379604d26b0d0654803513))


### Bug Fixes

* standardize error messages, add details ([#197](https://github.com/momentohq/client-sdk-rust/issues/197)) ([5fb1a23](https://github.com/momentohq/client-sdk-rust/commit/5fb1a231955606dd29b791ff4aeb142e893246e7))


### Miscellaneous

* add vscode recommended extensions ([#203](https://github.com/momentohq/client-sdk-rust/issues/203)) ([3d5da00](https://github.com/momentohq/client-sdk-rust/commit/3d5da000b32b3dc87bec67d7a10c4e6852b82acc))
* audit doctests for cache client creation ([#204](https://github.com/momentohq/client-sdk-rust/issues/204)) ([2e002ef](https://github.com/momentohq/client-sdk-rust/commit/2e002ef910aff4ef5038fa99906d22d4bfd273f2))
* fix minor docs warnings ([#198](https://github.com/momentohq/client-sdk-rust/issues/198)) ([1f86c71](https://github.com/momentohq/client-sdk-rust/commit/1f86c7156d67b6f25ae530909b5cfc8e031e5715))
* minor re-organization of integration tests and utils ([#193](https://github.com/momentohq/client-sdk-rust/issues/193)) ([5acc57d](https://github.com/momentohq/client-sdk-rust/commit/5acc57dc0531d159762f492d2e6504258d811f7f))
* move new sorted set fetch response type to new requests dir ([#195](https://github.com/momentohq/client-sdk-rust/issues/195)) ([0c8b4dc](https://github.com/momentohq/client-sdk-rust/commit/0c8b4dc4d5ebb6ed0ebcdc7e38d185ddbe6bb1ba))
* update CONTRIBUTING re: tests ([#196](https://github.com/momentohq/client-sdk-rust/issues/196)) ([75ded10](https://github.com/momentohq/client-sdk-rust/commit/75ded106023c9d32b223bb656ac0416a0a27a2c0))

## [0.33.1](https://github.com/momentohq/client-sdk-rust/compare/v0.33.0...v0.33.1) (2024-03-20)


### Miscellaneous

* try logging into crates.io before publishing ([#189](https://github.com/momentohq/client-sdk-rust/issues/189)) ([e5679c2](https://github.com/momentohq/client-sdk-rust/commit/e5679c24247f59ec666b1078e874e7d1b1b3c03f))

## [0.33.0](https://github.com/momentohq/client-sdk-rust/compare/v0.32.1...v0.33.0) (2024-03-20)


### Features

* add a configuration object for the cache client ([#176](https://github.com/momentohq/client-sdk-rust/issues/176)) ([2ee3227](https://github.com/momentohq/client-sdk-rust/commit/2ee3227a187fb6b0e13cacf9c5f9c7bace5489b2))
* add get/set to CacheClient to support readme.ts example ([#185](https://github.com/momentohq/client-sdk-rust/issues/185)) ([8d13be9](https://github.com/momentohq/client-sdk-rust/commit/8d13be9ae9b58bcaf84a58c17f13b1665377d2be))
* Add sorted set put and fetch functions ([#178](https://github.com/momentohq/client-sdk-rust/issues/178)) ([7a0d86b](https://github.com/momentohq/client-sdk-rust/commit/7a0d86ba67414088d0f411b2f48648c0d3d2bfe3))
* prototype of new API using requests as builders ([#175](https://github.com/momentohq/client-sdk-rust/issues/175)) ([33fd9f5](https://github.com/momentohq/client-sdk-rust/commit/33fd9f5f65a92ef8cb3891818e7a3b6eb4d86095))


### Miscellaneous

* disable releasing on every merge to main ([#172](https://github.com/momentohq/client-sdk-rust/issues/172)) ([6bb5965](https://github.com/momentohq/client-sdk-rust/commit/6bb596563c8ff0728746f4e2c4611118bda57541))
* fix and build examples ([#186](https://github.com/momentohq/client-sdk-rust/issues/186)) ([7023a10](https://github.com/momentohq/client-sdk-rust/commit/7023a10981d9c4c0c96b2bcbf539577235fb554e))
* modernize README, add CONTRIBUTING.md ([#182](https://github.com/momentohq/client-sdk-rust/issues/182)) ([188cf52](https://github.com/momentohq/client-sdk-rust/commit/188cf5217638047fb353afd0870da8c813beac1d))
* move prep_request out of SimpleCacheClient ([#174](https://github.com/momentohq/client-sdk-rust/issues/174)) ([5611dfb](https://github.com/momentohq/client-sdk-rust/commit/5611dfb0f65d7044e1b6f32dc95aad853430c246))
* remove next token from list_caches and list_signing_keys ([#173](https://github.com/momentohq/client-sdk-rust/issues/173)) ([009fe73](https://github.com/momentohq/client-sdk-rust/commit/009fe7363bb4e583e8b4a5a5b2ddaa2dc6dec74e))
* set up release please ([#187](https://github.com/momentohq/client-sdk-rust/issues/187)) ([f0d6dbb](https://github.com/momentohq/client-sdk-rust/commit/f0d6dbb287b2a09814a8dfe8b6283b9134777f67))
* standardize env var name to be MOMENTO_API_KEY ([#180](https://github.com/momentohq/client-sdk-rust/issues/180)) ([8b3b333](https://github.com/momentohq/client-sdk-rust/commit/8b3b333c5085ac9a66ac70a3d72dd4a97f064e6f))
