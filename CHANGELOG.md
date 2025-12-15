# Changelog

## [0.59.0](https://github.com/momentohq/client-sdk-rust/compare/v0.58.1...v0.59.0) (2025-12-15)


### Features

* new credential provider methods for accepting v2 api keys ([#511](https://github.com/momentohq/client-sdk-rust/issues/511)) ([dc8b7e2](https://github.com/momentohq/client-sdk-rust/commit/dc8b7e212f9d0a71d1f631dfa93d2141ed4d6275))

## [0.58.1](https://github.com/momentohq/client-sdk-rust/compare/v0.58.0...v0.58.1) (2025-11-18)


### Bug Fixes

* use latest protosocket libraries ([#509](https://github.com/momentohq/client-sdk-rust/issues/509)) ([f6df4d8](https://github.com/momentohq/client-sdk-rust/commit/f6df4d896fdc86898db736c6b1dabd63b081aa11))

## [0.58.0](https://github.com/momentohq/client-sdk-rust/compare/v0.57.0...v0.58.0) (2025-11-17)


### Features

* change default behavior for endpoints ([#505](https://github.com/momentohq/client-sdk-rust/issues/505)) ([2e10d68](https://github.com/momentohq/client-sdk-rust/commit/2e10d68ea726fc5fcee671829633a8e5dae0c9bd))


### Miscellaneous

* add protosocket cache client example ([#504](https://github.com/momentohq/client-sdk-rust/issues/504)) ([419df26](https://github.com/momentohq/client-sdk-rust/commit/419df261f4f6c755f2443af2c62866ab88073be0))
* **deps:** bump momento from 0.52.0 to 0.56.0 in /example/rust ([#499](https://github.com/momentohq/client-sdk-rust/issues/499)) ([3ed656e](https://github.com/momentohq/client-sdk-rust/commit/3ed656ea254a3933c6f42c11f201328c53da2d1b))
* updated protosocket example ([#507](https://github.com/momentohq/client-sdk-rust/issues/507)) ([b9f439b](https://github.com/momentohq/client-sdk-rust/commit/b9f439be340217e6d82f75ddd29f66fc3c63db69))

## [0.57.0](https://github.com/momentohq/client-sdk-rust/compare/v0.56.0...v0.57.0) (2025-10-31)


### Features

* add with_private_endpoints config option ([#500](https://github.com/momentohq/client-sdk-rust/issues/500)) ([d1efe2a](https://github.com/momentohq/client-sdk-rust/commit/d1efe2a29a8995ee5932ced71dff31eb1024419c))

## [0.56.0](https://github.com/momentohq/client-sdk-rust/compare/v0.55.0...v0.56.0) (2025-10-30)


### Features

* support tls endpoint override without address refresh ([#496](https://github.com/momentohq/client-sdk-rust/issues/496)) ([9984a57](https://github.com/momentohq/client-sdk-rust/commit/9984a5778a433f7be30b6d222eb9521a5ec412ad))

## [0.55.0](https://github.com/momentohq/client-sdk-rust/compare/v0.54.3...v0.55.0) (2025-10-20)


### Features

* configurable grpc message size limits ([#492](https://github.com/momentohq/client-sdk-rust/issues/492)) ([bdf3938](https://github.com/momentohq/client-sdk-rust/commit/bdf3938123dd27eee4fc4a51afc4259b70f3a50b))

## [0.54.3](https://github.com/momentohq/client-sdk-rust/compare/v0.54.2...v0.54.3) (2025-10-17)


### Bug Fixes

* protosocket connection manager should be able to connect to local endpoint too ([#490](https://github.com/momentohq/client-sdk-rust/issues/490)) ([d9963a5](https://github.com/momentohq/client-sdk-rust/commit/d9963a57ece1555643402f243f4cbf5bb724acd7))
* protosocket get should distinguish between cache vs key not found errors ([#489](https://github.com/momentohq/client-sdk-rust/issues/489)) ([ab54e24](https://github.com/momentohq/client-sdk-rust/commit/ab54e248c029dbe8fa18f3669931740642151a5b))

## [0.54.2](https://github.com/momentohq/client-sdk-rust/compare/v0.54.1...v0.54.2) (2025-10-09)


### Bug Fixes

* prevent early drop of address loader ([#486](https://github.com/momentohq/client-sdk-rust/issues/486)) ([405c861](https://github.com/momentohq/client-sdk-rust/commit/405c861a4e1aa1098db6ecee652171886f580680))

## [0.54.1](https://github.com/momentohq/client-sdk-rust/compare/v0.54.0...v0.54.1) (2025-10-06)


### Miscellaneous

* add direct connect handshake ([#483](https://github.com/momentohq/client-sdk-rust/issues/483)) ([e095bf9](https://github.com/momentohq/client-sdk-rust/commit/e095bf910487b96715219e892e259d96743afe50))

## [0.54.0](https://github.com/momentohq/client-sdk-rust/compare/v0.53.1...v0.54.0) (2025-10-01)


### Features

* protosocket with tls and connection pool ([#480](https://github.com/momentohq/client-sdk-rust/issues/480)) ([14fdc9a](https://github.com/momentohq/client-sdk-rust/commit/14fdc9a2de29d71915d5ec51f79f42082424c6f9))

## [0.53.1](https://github.com/momentohq/client-sdk-rust/compare/v0.53.0...v0.53.1) (2025-09-11)


### Bug Fixes

* be able to clone ProtosocketCacheClient ([#475](https://github.com/momentohq/client-sdk-rust/issues/475)) ([a72ff22](https://github.com/momentohq/client-sdk-rust/commit/a72ff22d3b081ed861304301361bc9e63729cbc4))

## [0.53.0](https://github.com/momentohq/client-sdk-rust/compare/v0.52.2...v0.53.0) (2025-09-11)


### Features

* protosocket cache client ([#473](https://github.com/momentohq/client-sdk-rust/issues/473)) ([207accc](https://github.com/momentohq/client-sdk-rust/commit/207accc0270e5d92b9e050864199251238fcc335))

## [0.52.2](https://github.com/momentohq/client-sdk-rust/compare/v0.52.1...v0.52.2) (2025-07-09)


### Miscellaneous

* add field accessors for Wasm and FunctionVersion ([#469](https://github.com/momentohq/client-sdk-rust/issues/469)) ([085c1af](https://github.com/momentohq/client-sdk-rust/commit/085c1afc6b728ed323be50e526b45fd25d43658f))

## [0.52.1](https://github.com/momentohq/client-sdk-rust/compare/v0.52.0...v0.52.1) (2025-07-08)


### Miscellaneous

* add Functions client ([#467](https://github.com/momentohq/client-sdk-rust/issues/467)) ([a9a0955](https://github.com/momentohq/client-sdk-rust/commit/a9a0955acb9935ab2de4c135bd6da30f811e47ed))
* add functions rpcs ([#468](https://github.com/momentohq/client-sdk-rust/issues/468)) ([1a795b7](https://github.com/momentohq/client-sdk-rust/commit/1a795b744e26a0727aca4af195a800e88abcb47a))
* **deps:** bump momento from 0.51.0 to 0.52.0 in /example/rust ([#464](https://github.com/momentohq/client-sdk-rust/issues/464)) ([47144e7](https://github.com/momentohq/client-sdk-rust/commit/47144e76e1989d236e941fc7b0350743c417bb64))
* update dependencies ([#466](https://github.com/momentohq/client-sdk-rust/issues/466)) ([6e14f24](https://github.com/momentohq/client-sdk-rust/commit/6e14f245edb8274460971599dbfe020148388ec4))

## [0.52.0](https://github.com/momentohq/client-sdk-rust/compare/v0.51.0...v0.52.0) (2025-05-13)


### Features

* add options for exclusive min/max for sorted set fetch by score and length by score ([#463](https://github.com/momentohq/client-sdk-rust/issues/463)) ([8681632](https://github.com/momentohq/client-sdk-rust/commit/868163291b101b557e46a24c2d6e70cc2836a223))


### Bug Fixes

* separate publish and subscribe channels in topic client ([#458](https://github.com/momentohq/client-sdk-rust/issues/458)) ([d083561](https://github.com/momentohq/client-sdk-rust/commit/d0835617c0468a035c0678991a119a8d776be0c2))

## [0.51.0](https://github.com/momentohq/client-sdk-rust/compare/v0.50.4...v0.51.0) (2025-05-05)


### Features

* add sorted_set_union_store and sorted_set_length_by_score + upgrades ([#456](https://github.com/momentohq/client-sdk-rust/issues/456)) ([49ab5ab](https://github.com/momentohq/client-sdk-rust/commit/49ab5ab661a30afe9454a236613c50b562cd5f0a))

## [0.50.4](https://github.com/momentohq/client-sdk-rust/compare/v0.50.3...v0.50.4) (2025-04-24)


### Miscellaneous

* export readytobuild ([#453](https://github.com/momentohq/client-sdk-rust/issues/453)) ([4ceab52](https://github.com/momentohq/client-sdk-rust/commit/4ceab5240f5d847fc9a46895e4a7da5bf80bf190))

## [0.50.3](https://github.com/momentohq/client-sdk-rust/compare/v0.50.2...v0.50.3) (2025-04-24)


### Miscellaneous

* export cache client builder ([#450](https://github.com/momentohq/client-sdk-rust/issues/450)) ([01ee675](https://github.com/momentohq/client-sdk-rust/commit/01ee675c3f0461409adb2183c75ed65ec7c1ace4))

## [0.50.2](https://github.com/momentohq/client-sdk-rust/compare/v0.50.1...v0.50.2) (2025-04-24)


### Miscellaneous

* add basic trait derivations for cache client builder ([#448](https://github.com/momentohq/client-sdk-rust/issues/448)) ([9d510e8](https://github.com/momentohq/client-sdk-rust/commit/9d510e8f624a98b5970f68b671322717842ebe3f))

## [0.50.1](https://github.com/momentohq/client-sdk-rust/compare/v0.50.0...v0.50.1) (2025-04-23)


### Miscellaneous

* add more from conversions and refactor to make less errorful ([#442](https://github.com/momentohq/client-sdk-rust/issues/442)) ([7692abe](https://github.com/momentohq/client-sdk-rust/commit/7692abe8bdefa5daed11ef9ebb53f7f303887732))

## [0.50.0](https://github.com/momentohq/client-sdk-rust/compare/v0.49.0...v0.50.0) (2025-04-23)


### Features

* remove storage client ([#443](https://github.com/momentohq/client-sdk-rust/issues/443)) ([fb508a1](https://github.com/momentohq/client-sdk-rust/commit/fb508a18abeece6380ffc5ca66ac23f3085c4ef3))

## [0.49.0](https://github.com/momentohq/client-sdk-rust/compare/v0.48.0...v0.49.0) (2025-02-12)


### Features

* add competition ranking to leaderboard ([#439](https://github.com/momentohq/client-sdk-rust/issues/439)) ([df4bd46](https://github.com/momentohq/client-sdk-rust/commit/df4bd4675e8398ff31dbe61d56935c2dfad9f687))

## [0.48.0](https://github.com/momentohq/client-sdk-rust/compare/v0.47.1...v0.48.0) (2025-02-11)


### Features

* momento leaderboard client ([#438](https://github.com/momentohq/client-sdk-rust/issues/438)) ([1223adc](https://github.com/momentohq/client-sdk-rust/commit/1223adcd25b358c939752b05c4a2e0fcd377d832))


### Miscellaneous

* **deps:** bump momento from 0.47.0 to 0.47.1 in /example/rust ([#433](https://github.com/momentohq/client-sdk-rust/issues/433)) ([08b5664](https://github.com/momentohq/client-sdk-rust/commit/08b566417dfb4765ddb1ec9e44062520a936cb73))
* update topics example to use oneshot synchronization instead ([#430](https://github.com/momentohq/client-sdk-rust/issues/430)) ([1eb846d](https://github.com/momentohq/client-sdk-rust/commit/1eb846d6c0b6047aaa4622382449a5eef3fd8afd))
* update topics example to use updated auth client UX ([#434](https://github.com/momentohq/client-sdk-rust/issues/434)) ([7727e3f](https://github.com/momentohq/client-sdk-rust/commit/7727e3ffcb8204f4b7c3bea606daa9a34f94548a))

## [0.47.1](https://github.com/momentohq/client-sdk-rust/compare/v0.47.0...v0.47.1) (2025-01-09)


### Bug Fixes

* add permission scope unit tests and From conversions for convenience ([#427](https://github.com/momentohq/client-sdk-rust/issues/427)) ([dbfdf6d](https://github.com/momentohq/client-sdk-rust/commit/dbfdf6d3cd2cb4c8911936bf02f51076fc839f6e))


### Miscellaneous

* add auth client docs snippets ([#424](https://github.com/momentohq/client-sdk-rust/issues/424)) ([477795c](https://github.com/momentohq/client-sdk-rust/commit/477795c755a2e88928e296b07238eea82d3cb315))
* add unit tests for expiration objects and expand all_data_read_write disposable token test for completeness ([#425](https://github.com/momentohq/client-sdk-rust/issues/425)) ([feb9ec4](https://github.com/momentohq/client-sdk-rust/commit/feb9ec41f4b80a50c00bf53d18bb01f205969141))
* showcase publisher_id in topics example ([#418](https://github.com/momentohq/client-sdk-rust/issues/418)) ([0f06884](https://github.com/momentohq/client-sdk-rust/commit/0f06884778d67907c022df10f82995301315e35b))

## [0.47.0](https://github.com/momentohq/client-sdk-rust/compare/v0.46.1...v0.47.0) (2025-01-03)


### Features

* add auth client and generate_disposable_token api ([#420](https://github.com/momentohq/client-sdk-rust/issues/420)) ([432babf](https://github.com/momentohq/client-sdk-rust/commit/432babf2d708e052fd6d6c87f7c1bc9a33a9cfed))


### Miscellaneous

* update vscode workspace to match directory structure ([#417](https://github.com/momentohq/client-sdk-rust/issues/417)) ([f28a368](https://github.com/momentohq/client-sdk-rust/commit/f28a36858b33ba2adc1ee0c57a4aa1c1f7d16997))

## [0.46.1](https://github.com/momentohq/client-sdk-rust/compare/v0.46.0...v0.46.1) (2024-12-16)


### Miscellaneous

* remove dead code to set cargo version ([#413](https://github.com/momentohq/client-sdk-rust/issues/413)) ([771d4b8](https://github.com/momentohq/client-sdk-rust/commit/771d4b80d595de51bb72c590eb125fae6318debc))
* update cargo lock ([#411](https://github.com/momentohq/client-sdk-rust/issues/411)) ([f8b7a38](https://github.com/momentohq/client-sdk-rust/commit/f8b7a38f941d9510536a016c79db611aa71500ba))

## [0.46.0](https://github.com/momentohq/client-sdk-rust/compare/v0.45.0...v0.46.0) (2024-12-13)


### Features

* fix ci for release ([#408](https://github.com/momentohq/client-sdk-rust/issues/408)) ([8b3cfea](https://github.com/momentohq/client-sdk-rust/commit/8b3cfea9c9049748e3364e86d1549f67af880e43))

## [0.45.0](https://github.com/momentohq/client-sdk-rust/compare/v0.44.0...v0.45.0) (2024-12-12)


### Features

* add get_batch and set_batch ([#406](https://github.com/momentohq/client-sdk-rust/issues/406)) ([d0ac9e7](https://github.com/momentohq/client-sdk-rust/commit/d0ac9e70899a48340e67360df3a8495941fd5bf6))


### Miscellaneous

* **deps:** bump momento from 0.43.0 to 0.44.0 in /example/rust ([#402](https://github.com/momentohq/client-sdk-rust/issues/402)) ([747bd9a](https://github.com/momentohq/client-sdk-rust/commit/747bd9ac839487e910afffa29d3c72810cb15618))
* remove deprecated crates extension ([#404](https://github.com/momentohq/client-sdk-rust/issues/404)) ([d81a66a](https://github.com/momentohq/client-sdk-rust/commit/d81a66aef7e726aaf24e6c2e496feccb7c421afa))
* sdk/Cargo.lock should use correct version of momento packages ([#405](https://github.com/momentohq/client-sdk-rust/issues/405)) ([bcd74ed](https://github.com/momentohq/client-sdk-rust/commit/bcd74ed5418468e29d1aa4e88fe9513f0007b304))

## [0.44.0](https://github.com/momentohq/client-sdk-rust/compare/v0.43.1...v0.44.0) (2024-11-21)


### Features

* add sequence page to topics ([#400](https://github.com/momentohq/client-sdk-rust/issues/400)) ([29ee341](https://github.com/momentohq/client-sdk-rust/commit/29ee3411716461c981ca78241d64abf1a9efe693))

## [0.43.1](https://github.com/momentohq/client-sdk-rust/compare/v0.43.0...v0.43.1) (2024-11-08)


### Miscellaneous

* **deps-dev:** bump aws-cdk in /example/aws/cdk-lambda/infrastructure ([#395](https://github.com/momentohq/client-sdk-rust/issues/395)) ([6090dc8](https://github.com/momentohq/client-sdk-rust/commit/6090dc8ae457e765fdaaa3306af45da6dc5061fe))
* **deps:** bump momento from 0.42.0 to 0.43.0 in /example/rust ([#394](https://github.com/momentohq/client-sdk-rust/issues/394)) ([1ae2a4d](https://github.com/momentohq/client-sdk-rust/commit/1ae2a4d88cc6b0dac3d2c6b07a82718636e076e9))
* fix broken links in readmes ([#397](https://github.com/momentohq/client-sdk-rust/issues/397)) ([b07c4ad](https://github.com/momentohq/client-sdk-rust/commit/b07c4adc93d4db3d36da37b13867f989df148587))
* improve resource exhausted message ([#398](https://github.com/momentohq/client-sdk-rust/issues/398)) ([84a0cbf](https://github.com/momentohq/client-sdk-rust/commit/84a0cbf9398e91f429a850b8469f511296171c87))

## [0.43.0](https://github.com/momentohq/client-sdk-rust/compare/v0.42.0...v0.43.0) (2024-10-09)


### Features

* adding support for multiple connections ([#393](https://github.com/momentohq/client-sdk-rust/issues/393)) ([2e79c87](https://github.com/momentohq/client-sdk-rust/commit/2e79c878830ee4c7789f7caa45ecbedbc836b0c6))


### Miscellaneous

* add dev docs snippets and cheat sheet for storage client ([#389](https://github.com/momentohq/client-sdk-rust/issues/389)) ([223be86](https://github.com/momentohq/client-sdk-rust/commit/223be86afab4f5968660b925c674be133b693c28))
* **deps:** bump momento from 0.41.3 to 0.42.0 in /example/rust ([#387](https://github.com/momentohq/client-sdk-rust/issues/387)) ([e64afce](https://github.com/momentohq/client-sdk-rust/commit/e64afceb4198c3fb4eb6ddd0afbf40b329c552d4))

## [0.42.0](https://github.com/momentohq/client-sdk-rust/compare/v0.41.3...v0.42.0) (2024-07-12)


### Features

* add example for preview storage client ([#368](https://github.com/momentohq/client-sdk-rust/issues/368)) ([61ad15e](https://github.com/momentohq/client-sdk-rust/commit/61ad15e89d139ed25af53b42a16234f2b6a4435d))


### Bug Fixes

* interceptor should send only once per client ([#384](https://github.com/momentohq/client-sdk-rust/issues/384)) ([8ed2e86](https://github.com/momentohq/client-sdk-rust/commit/8ed2e86d060fdf0da7d771a67dfcd3fad5499b2a))
* publisher_id was missing from received subscription items ([#385](https://github.com/momentohq/client-sdk-rust/issues/385)) ([1fc1804](https://github.com/momentohq/client-sdk-rust/commit/1fc1804538aa1cce669c8ef7b0901a393d9834a2))


### Miscellaneous

* add vscode workspace ([#383](https://github.com/momentohq/client-sdk-rust/issues/383)) ([b3d219b](https://github.com/momentohq/client-sdk-rust/commit/b3d219b8e2be74045d6cf93c1440de9cd36ae618))
* send sentinel value for `runtime-version` header ([#381](https://github.com/momentohq/client-sdk-rust/issues/381)) ([56401d8](https://github.com/momentohq/client-sdk-rust/commit/56401d8676abf9721cb7d579a6eb8b3fdf8baa36))
* update examples momento dep and update storage example get responses ([#386](https://github.com/momentohq/client-sdk-rust/issues/386)) ([6d3ee17](https://github.com/momentohq/client-sdk-rust/commit/6d3ee1743d86dba55658bddbbb9216548c41f033))

## [0.41.3](https://github.com/momentohq/client-sdk-rust/compare/v0.41.2...v0.41.3) (2024-07-05)


### Bug Fixes

* storage get response should use Found and NotFound cases ([#376](https://github.com/momentohq/client-sdk-rust/issues/376)) ([8f3344f](https://github.com/momentohq/client-sdk-rust/commit/8f3344f78fc907c6c93cd5a15f616eca513f151e))

## [0.41.2](https://github.com/momentohq/client-sdk-rust/compare/v0.41.1...v0.41.2) (2024-07-03)


### Miscellaneous

* adjust agent header value per spec ([#375](https://github.com/momentohq/client-sdk-rust/issues/375)) ([610ddf9](https://github.com/momentohq/client-sdk-rust/commit/610ddf9287dd825b4d716a2bcc2a1508a5d2f176))

## [0.41.1](https://github.com/momentohq/client-sdk-rust/compare/v0.41.0...v0.41.1) (2024-07-03)


### Bug Fixes

* export StorageValue and align the exports with the cache client. ([#370](https://github.com/momentohq/client-sdk-rust/issues/370)) ([e75608f](https://github.com/momentohq/client-sdk-rust/commit/e75608f919892fb24a7cbadd822748f91300e5cb))


### Miscellaneous

* **deps:** bump momento from 0.39.7 to 0.41.0 in /example/rust ([#369](https://github.com/momentohq/client-sdk-rust/issues/369)) ([08f7828](https://github.com/momentohq/client-sdk-rust/commit/08f78285ce947f6a6fc2bd5bf8072a2d5573b778))
* fix failing test for StoreNotFound ([#374](https://github.com/momentohq/client-sdk-rust/issues/374)) ([e5990de](https://github.com/momentohq/client-sdk-rust/commit/e5990deea83c05ce39e09c820a6b258f433489b7))

## [0.41.0](https://github.com/momentohq/client-sdk-rust/compare/v0.40.0...v0.41.0) (2024-06-21)


### Features

* Add a preview Storage Client ([#354](https://github.com/momentohq/client-sdk-rust/issues/354)) ([8aa9b7c](https://github.com/momentohq/client-sdk-rust/commit/8aa9b7c62e1eb078db157b1bb7485d757d1d5a47))


### Miscellaneous

* remove legacy jwt token support ([#366](https://github.com/momentohq/client-sdk-rust/issues/366)) ([ba90ff0](https://github.com/momentohq/client-sdk-rust/commit/ba90ff072664376051807c44d5aba4824d523800))
* remove unused read concern mod ([#363](https://github.com/momentohq/client-sdk-rust/issues/363)) ([86f876b](https://github.com/momentohq/client-sdk-rust/commit/86f876bda6c737ca87ace2a61e71f3a3ddb50fcb))

## [0.40.0](https://github.com/momentohq/client-sdk-rust/compare/v0.39.7...v0.40.0) (2024-06-14)


### Features

* add sorted_set_get_scores and sorted_set_increment methods ([#360](https://github.com/momentohq/client-sdk-rust/issues/360)) ([ba95599](https://github.com/momentohq/client-sdk-rust/commit/ba9559941baa547c378f39ad3fa63f7b494907e8))

## [0.39.7](https://github.com/momentohq/client-sdk-rust/compare/v0.39.6...v0.39.7) (2024-06-06)


### Bug Fixes

* Move tests directory into sdk ([#355](https://github.com/momentohq/client-sdk-rust/issues/355)) ([3d7654b](https://github.com/momentohq/client-sdk-rust/commit/3d7654b9d798db2c526df2bc67fc8352f9c739c4))


### Miscellaneous

* add CDK and zip lambda examples ([#342](https://github.com/momentohq/client-sdk-rust/issues/342)) ([6ff7e97](https://github.com/momentohq/client-sdk-rust/commit/6ff7e97cd87bb20e8f47ee060909170bd433d5bd))
* make optional things optional on request builders ([#357](https://github.com/momentohq/client-sdk-rust/issues/357)) ([62c058e](https://github.com/momentohq/client-sdk-rust/commit/62c058edbfd40f4dd8777148a294de7585948c9c)), closes [#356](https://github.com/momentohq/client-sdk-rust/issues/356)

## [0.39.6](https://github.com/momentohq/client-sdk-rust/compare/v0.39.5...v0.39.6) (2024-05-23)


### Miscellaneous

* trying to fix release-please; remove path, add extra-files ([#350](https://github.com/momentohq/client-sdk-rust/issues/350)) ([e1fc5ec](https://github.com/momentohq/client-sdk-rust/commit/e1fc5ec3e9174600329fa7d5df5a221c9cca79a9))

## [0.39.3](https://github.com/momentohq/client-sdk-rust/compare/v0.39.2...v0.39.3) (2024-05-23)


### Miscellaneous

* add sdk/README to gitignore to prevent attempt to publish from dirty tree ([#343](https://github.com/momentohq/client-sdk-rust/issues/343)) ([8298189](https://github.com/momentohq/client-sdk-rust/commit/82981895de7e125dd4f79cf307836c8ea890d46d))

## [0.39.2](https://github.com/momentohq/client-sdk-rust/compare/v0.39.1...v0.39.2) (2024-05-22)


### Bug Fixes

* publish task broken with sdk subdir ([#340](https://github.com/momentohq/client-sdk-rust/issues/340)) ([a2e42cc](https://github.com/momentohq/client-sdk-rust/commit/a2e42cc9f795c5e108fa5df66c9535b979285268))


### Miscellaneous

* rename github workflows to be less confusing ([#339](https://github.com/momentohq/client-sdk-rust/issues/339)) ([d15df74](https://github.com/momentohq/client-sdk-rust/commit/d15df744d7e474d0cae99eb40f4f77e4cdeb0af4))

## [0.39.1](https://github.com/momentohq/client-sdk-rust/compare/v0.39.0...v0.39.1) (2024-05-22)


### Miscellaneous

* **deps:** update momento requirement from 0.38.0 to 0.39.0 in /example ([#331](https://github.com/momentohq/client-sdk-rust/issues/331)) ([7bf0f16](https://github.com/momentohq/client-sdk-rust/commit/7bf0f166b5356a436045853f96843283ebb7b684))
* implement Debug for DictionaryFetchResponse, tweak debug fmt utils ([#335](https://github.com/momentohq/client-sdk-rust/issues/335)) ([37dd7a9](https://github.com/momentohq/client-sdk-rust/commit/37dd7a93fcc3145b7be1f822cd439f578b624e91))
* move existing examples to a subdirectory ([#337](https://github.com/momentohq/client-sdk-rust/issues/337)) ([eb16864](https://github.com/momentohq/client-sdk-rust/commit/eb16864a3a9482704c370fb69e895c67a4e46876))
* move sdk source code to a subdir ([#338](https://github.com/momentohq/client-sdk-rust/issues/338)) ([8eddf9a](https://github.com/momentohq/client-sdk-rust/commit/8eddf9a96d63c12da630da82aca9fe89bac216ce))
* run ci on ubuntu-24.04 ([#332](https://github.com/momentohq/client-sdk-rust/issues/332)) ([8338069](https://github.com/momentohq/client-sdk-rust/commit/8338069551493f87a4a60559b8ff95fef4f45d7b))
* update stability to beta ([#336](https://github.com/momentohq/client-sdk-rust/issues/336)) ([ebf044f](https://github.com/momentohq/client-sdk-rust/commit/ebf044f757237f876cf362cfa8803e10d3092b6b))

## [0.39.0](https://github.com/momentohq/client-sdk-rust/compare/v0.38.0...v0.39.0) (2024-05-15)


### Features

* add list_push_front and list_push_back ([#294](https://github.com/momentohq/client-sdk-rust/issues/294)) ([ace68b8](https://github.com/momentohq/client-sdk-rust/commit/ace68b8315b47bbe9f393bc9a47c64b354ce607e))
* add response suffix to control commands ([#313](https://github.com/momentohq/client-sdk-rust/issues/313)) ([1c197d4](https://github.com/momentohq/client-sdk-rust/commit/1c197d4f5a77b3549a3ddebd52829087a3273776))
* add response suffix to dictionary response types ([#300](https://github.com/momentohq/client-sdk-rust/issues/300)) ([1f4c80b](https://github.com/momentohq/client-sdk-rust/commit/1f4c80bdb9d26b97d2a2a3d90dc8a837c055c4b2))
* add response suffix to list types ([#305](https://github.com/momentohq/client-sdk-rust/issues/305)) ([da538af](https://github.com/momentohq/client-sdk-rust/commit/da538afaaf22cf95b16783343100a9895c0d4f78))
* add response suffix to scalar types ([#307](https://github.com/momentohq/client-sdk-rust/issues/307)) ([51bd289](https://github.com/momentohq/client-sdk-rust/commit/51bd289b42ba46901f076bf7f4725cdcfe2222f9))
* add response suffix to set type ([#311](https://github.com/momentohq/client-sdk-rust/issues/311)) ([374b499](https://github.com/momentohq/client-sdk-rust/commit/374b49958313b667f94beec598eef70b589aaf83))
* add response suffix to sorted set types ([#312](https://github.com/momentohq/client-sdk-rust/issues/312)) ([70d45ac](https://github.com/momentohq/client-sdk-rust/commit/70d45ac636bdee3795f8dba844e38154173fbb94))
* basic subscriptions should return only SubscriptionValues ([#316](https://github.com/momentohq/client-sdk-rust/issues/316)) ([789cf02](https://github.com/momentohq/client-sdk-rust/commit/789cf02ddd0547a7c3a8b06742b282baf20f5008))
* implement custom debug and display traits for byte-array types ([#323](https://github.com/momentohq/client-sdk-rust/issues/323)) ([4eaf53d](https://github.com/momentohq/client-sdk-rust/commit/4eaf53dcae4da48e7964d02a0d0c0b9036610337)), closes [#282](https://github.com/momentohq/client-sdk-rust/issues/282)


### Bug Fixes

* clippy error on gh with doctest signature ([#325](https://github.com/momentohq/client-sdk-rust/issues/325)) ([4b75542](https://github.com/momentohq/client-sdk-rust/commit/4b755425ebcaf3de79d40123c5afa3bacda801c3)), closes [#326](https://github.com/momentohq/client-sdk-rust/issues/326)
* impl Display for CredentialProvider ([#295](https://github.com/momentohq/client-sdk-rust/issues/295)) ([ade83f2](https://github.com/momentohq/client-sdk-rust/commit/ade83f2c8b72afca1bd8d09903eee694aca8fa13))


### Miscellaneous

* Add / tweak doc examples for use in dev docs ([#317](https://github.com/momentohq/client-sdk-rust/issues/317)) ([6e4de64](https://github.com/momentohq/client-sdk-rust/commit/6e4de64d4ef459e83d175a6bc35a74fd1d16a749))
* add docs to improve discoverability for client instantiation ([#320](https://github.com/momentohq/client-sdk-rust/issues/320)) ([e883d20](https://github.com/momentohq/client-sdk-rust/commit/e883d20cade2ebf674da7af542293c6c672539fa))
* add integration tests and doctests for TopicClient ([#292](https://github.com/momentohq/client-sdk-rust/issues/292)) ([a93b319](https://github.com/momentohq/client-sdk-rust/commit/a93b319d2fe17ea5dcb673d4874b4ba3b77bd78d))
* add links to response docstrings showing how to handle hits/misses ([#318](https://github.com/momentohq/client-sdk-rust/issues/318)) ([9af89af](https://github.com/momentohq/client-sdk-rust/commit/9af89afe9dfacf3cc776a269d14b85283d601a2f))
* add lint rule to check usage of expect ([#293](https://github.com/momentohq/client-sdk-rust/issues/293)) ([be547e4](https://github.com/momentohq/client-sdk-rust/commit/be547e42964ed958ff943f1da084a2f78d4bf379))
* add missing links in docs, small docs fixes ([#301](https://github.com/momentohq/client-sdk-rust/issues/301)) ([14e909e](https://github.com/momentohq/client-sdk-rust/commit/14e909e93ddce89fae45d3de1fd6009247b4df40))
* add more docstrings for errors and utils, remove unused file ([#310](https://github.com/momentohq/client-sdk-rust/issues/310)) ([7e46916](https://github.com/momentohq/client-sdk-rust/commit/7e4691609f46d0ebba216ef44b61e1178bbb4770))
* add top-level docs for the crate, including 'into' conventions etc. ([#304](https://github.com/momentohq/client-sdk-rust/issues/304)) ([e6d9ebe](https://github.com/momentohq/client-sdk-rust/commit/e6d9ebe25a5e2123f331a469e5235061c14a4682))
* enable missing_docs lint rule and fill in missing docs ([#321](https://github.com/momentohq/client-sdk-rust/issues/321)) ([ce4a1ed](https://github.com/momentohq/client-sdk-rust/commit/ce4a1edf7534b90f6bc2d796ddecf5311afdf130))
* improve error messages in tests ([#322](https://github.com/momentohq/client-sdk-rust/issues/322)) ([db927a2](https://github.com/momentohq/client-sdk-rust/commit/db927a2f8c5b34013c7f333be0fdb05bb8703e46))
* move docs examples into subdir ([#319](https://github.com/momentohq/client-sdk-rust/issues/319)) ([1300e83](https://github.com/momentohq/client-sdk-rust/commit/1300e8379a9a59efcc7dd0bd6dd15bb3a178ac8a))
* remove obsolete protoc scripts and ci references ([#328](https://github.com/momentohq/client-sdk-rust/issues/328)) ([dc0a3f0](https://github.com/momentohq/client-sdk-rust/commit/dc0a3f0eba5836f7b700dcb44af67d8adbcf5478))
* Templatize example README, expand to include topics example ([#309](https://github.com/momentohq/client-sdk-rust/issues/309)) ([54a181b](https://github.com/momentohq/client-sdk-rust/commit/54a181b0c79110c75598b08924c75d674785359a))
* upgrade codeql action upload sarif version ([#330](https://github.com/momentohq/client-sdk-rust/issues/330)) ([e09d933](https://github.com/momentohq/client-sdk-rust/commit/e09d933a1ce763d9121483fd3baf9710224cfd72))

## [0.38.0](https://github.com/momentohq/client-sdk-rust/compare/v0.37.0...v0.38.0) (2024-05-10)


### Features

* TopicClient follow-up revisions" ([#287](https://github.com/momentohq/client-sdk-rust/issues/287)) ([66e190d](https://github.com/momentohq/client-sdk-rust/commit/66e190d1bea27d4feb3ceabaa856af53d8274e47))


### Miscellaneous

* allow `IntoBytesIterable` on statically allocated arrays ([#288](https://github.com/momentohq/client-sdk-rust/issues/288)) ([82c556c](https://github.com/momentohq/client-sdk-rust/commit/82c556c424d3fe37e72faf771667a8245ca92242))

## [0.37.0](https://github.com/momentohq/client-sdk-rust/compare/v0.36.0...v0.37.0) (2024-05-09)


### Features

* update the topic client for consistency with cache client ([#276](https://github.com/momentohq/client-sdk-rust/issues/276)) ([fd5d5a8](https://github.com/momentohq/client-sdk-rust/commit/fd5d5a8ae4bd1860c85c5aa7cd195eacf9c41c1d))


### Miscellaneous

* add settings file for rust analyzer env variables ([#268](https://github.com/momentohq/client-sdk-rust/issues/268)) ([a084f75](https://github.com/momentohq/client-sdk-rust/commit/a084f75923754745fc4a11efd3ba4842e535ef29))
* flush_cache test should create own cache ([#272](https://github.com/momentohq/client-sdk-rust/issues/272)) ([c55079e](https://github.com/momentohq/client-sdk-rust/commit/c55079e2a288e887a4e2ba7f0a1591d6161ba8cc))

## [0.36.0](https://github.com/momentohq/client-sdk-rust/compare/v0.35.0...v0.36.0) (2024-05-07)


### Features

* remove legacy sdk and related artifacts ([#265](https://github.com/momentohq/client-sdk-rust/issues/265)) ([e8221ab](https://github.com/momentohq/client-sdk-rust/commit/e8221abc85e164555144c4e191c7ca013f84b852))


### Miscellaneous

* audit docs for usage of key/field/value ([#266](https://github.com/momentohq/client-sdk-rust/issues/266)) ([71016e2](https://github.com/momentohq/client-sdk-rust/commit/71016e2e44641d01a22d436ded26b3f56066da39))
* port example to new sdk ([#262](https://github.com/momentohq/client-sdk-rust/issues/262)) ([fd240b6](https://github.com/momentohq/client-sdk-rust/commit/fd240b6c386ee962d1db29a52d4152eeddc375e8))
* verify examples in CI and clean up workflow files ([#264](https://github.com/momentohq/client-sdk-rust/issues/264)) ([a34e4ee](https://github.com/momentohq/client-sdk-rust/commit/a34e4eeb0824229bc2e32d01b6e37b53ffacd6ce))

## [0.35.0](https://github.com/momentohq/client-sdk-rust/compare/v0.34.0...v0.35.0) (2024-05-06)


### Features

* add `IntoSortedSetElements` trait and implementations for Vec and HashMap ([#207](https://github.com/momentohq/client-sdk-rust/issues/207)) ([aaf4063](https://github.com/momentohq/client-sdk-rust/commit/aaf4063067b7a3428b647caf186b6208aea15d1c))
* add Delete, Increment, ItemGetType ([#219](https://github.com/momentohq/client-sdk-rust/issues/219)) ([278adda](https://github.com/momentohq/client-sdk-rust/commit/278addae4ba39715179a7447e56a356374310b16))
* add dictionary_set_fields ([#244](https://github.com/momentohq/client-sdk-rust/issues/244)) ([15d8a2f](https://github.com/momentohq/client-sdk-rust/commit/15d8a2fdc8aab7d1430ccd5534272cdf9aef849f))
* add FlushCache and tests ([#209](https://github.com/momentohq/client-sdk-rust/issues/209)) ([d8da447](https://github.com/momentohq/client-sdk-rust/commit/d8da4477668a43ef87f306c7f7032efa0815b009))
* add ItemGet/Update/Increase/Decrease TTL APIs ([#226](https://github.com/momentohq/client-sdk-rust/issues/226)) ([05886be](https://github.com/momentohq/client-sdk-rust/commit/05886be9bf0d187db426bf16d4abf130b8a11eaf))
* add keyExists and keysExist ([#217](https://github.com/momentohq/client-sdk-rust/issues/217)) ([6b167a2](https://github.com/momentohq/client-sdk-rust/commit/6b167a2f95f139f974811664fe2a365d93b5482c))
* add list collection ([#237](https://github.com/momentohq/client-sdk-rust/issues/237)) ([27279fa](https://github.com/momentohq/client-sdk-rust/commit/27279faaa35092d16244371780b851213d6759bc))
* add more sorted set methods ([#243](https://github.com/momentohq/client-sdk-rust/issues/243)) ([00fcc91](https://github.com/momentohq/client-sdk-rust/commit/00fcc914355cc1af18b7e9d4a4d4bb7fe0d82357))
* add set_fetch and set_remove_elements ([#248](https://github.com/momentohq/client-sdk-rust/issues/248)) ([7ad34e0](https://github.com/momentohq/client-sdk-rust/commit/7ad34e0f339b9815dd2f9d84a03a9e4faab84e67))
* add SetIf* methods ([#234](https://github.com/momentohq/client-sdk-rust/issues/234)) ([f812c63](https://github.com/momentohq/client-sdk-rust/commit/f812c6397f551a553d23155da6fb0bec16bf8abe))
* add start_rank and end_rank as optional arguments to SortedSetFetchByRank ([#215](https://github.com/momentohq/client-sdk-rust/issues/215)) ([5bece54](https://github.com/momentohq/client-sdk-rust/commit/5bece54b57928ca5f1ad966c11b1b6bef25c3ed7))
* dictionary_fetch and dictionary_set_field implementation ([#236](https://github.com/momentohq/client-sdk-rust/issues/236)) ([9cdabb6](https://github.com/momentohq/client-sdk-rust/commit/9cdabb6436a0455652f57d2b0a70e703e6530801))
* implement dictionary remove fields ([#249](https://github.com/momentohq/client-sdk-rust/issues/249)) ([c808a33](https://github.com/momentohq/client-sdk-rust/commit/c808a3380db0df2c34be08dcb452ae32e68689b0))
* implement dictionary_get_field ([#254](https://github.com/momentohq/client-sdk-rust/issues/254)) ([d5ecaf6](https://github.com/momentohq/client-sdk-rust/commit/d5ecaf6a7b463136873c94e5223a0799f971ca54))
* implement dictionary_get_field ([#255](https://github.com/momentohq/client-sdk-rust/issues/255)) ([519852a](https://github.com/momentohq/client-sdk-rust/commit/519852a4bec12b1a0bff61566249fe49284a88d7))
* implement dictionary_get_fields ([#247](https://github.com/momentohq/client-sdk-rust/issues/247)) ([087b3c4](https://github.com/momentohq/client-sdk-rust/commit/087b3c430e9bf01be625a444b12736501045ea3b))
* implement dictionary_increment ([#256](https://github.com/momentohq/client-sdk-rust/issues/256)) ([6b2d69c](https://github.com/momentohq/client-sdk-rust/commit/6b2d69cf092f7945a33689575acdf4c1c53a7a18))
* implement dictionary_length ([#250](https://github.com/momentohq/client-sdk-rust/issues/250)) ([903509f](https://github.com/momentohq/client-sdk-rust/commit/903509f6b58828854d11f67fb41f8680982101c2))
* re-export CollectionTtl under cache namespace ([#228](https://github.com/momentohq/client-sdk-rust/issues/228)) ([8a8a93e](https://github.com/momentohq/client-sdk-rust/commit/8a8a93e8d250d5a45c5bb033b889673aa1372696))
* reorganize exports ([#221](https://github.com/momentohq/client-sdk-rust/issues/221)) ([6fada71](https://github.com/momentohq/client-sdk-rust/commit/6fada71e1160976da10498ab47e8db98511fa437))
* reorganize topics exports ([#224](https://github.com/momentohq/client-sdk-rust/issues/224)) ([b3bde84](https://github.com/momentohq/client-sdk-rust/commit/b3bde84ca17ee91bf55ecdc2881a04e4af57e914))


### Bug Fixes

* increment should refer to key as key not field ([#257](https://github.com/momentohq/client-sdk-rust/issues/257)) ([6518427](https://github.com/momentohq/client-sdk-rust/commit/651842732ae78a3ae392e075ad93ba4747ac6fa8))
* move prep_request_with_timeout into utils ([#223](https://github.com/momentohq/client-sdk-rust/issues/223)) ([b9a68af](https://github.com/momentohq/client-sdk-rust/commit/b9a68af2124a7fde91543a9e448af68661588f4e))
* replace usage of unreachable macro with returning an UnknownError ([#242](https://github.com/momentohq/client-sdk-rust/issues/242)) ([1083717](https://github.com/momentohq/client-sdk-rust/commit/108371702273a5763046a3719f2d67e27ca33fd8))


### Miscellaneous

* add snippets for dev docs, corrected some docstring examples in cache client ([#253](https://github.com/momentohq/client-sdk-rust/issues/253)) ([834a1c5](https://github.com/momentohq/client-sdk-rust/commit/834a1c5d6664f9c97b107b56349cbd1a39cc0896))
* extract lists of IntoBytes into an IntoBytesIterable trait ([#252](https://github.com/momentohq/client-sdk-rust/issues/252)) ([ac81f32](https://github.com/momentohq/client-sdk-rust/commit/ac81f321cccbdde41f04679966d83857ffc786f0))
* fix minor issues in makefile and github actions ([#211](https://github.com/momentohq/client-sdk-rust/issues/211)) ([73f2284](https://github.com/momentohq/client-sdk-rust/commit/73f2284a2bb6fab303581bb7c9d00baa8886ef7b))
* improve docs, replace asserts with pattern matching, reduce duplication of code examples ([#213](https://github.com/momentohq/client-sdk-rust/issues/213)) ([0df51ee](https://github.com/momentohq/client-sdk-rust/commit/0df51ee69c1d1058452a27b81a9943a6d2d827ae))
* make build fail when there are docs warnings ([#214](https://github.com/momentohq/client-sdk-rust/issues/214)) ([f8a82f0](https://github.com/momentohq/client-sdk-rust/commit/f8a82f0d6b90e4d0cb732efb347a4c2589dc60c9))
* minor test cleanups ([#225](https://github.com/momentohq/client-sdk-rust/issues/225)) ([ba18c65](https://github.com/momentohq/client-sdk-rust/commit/ba18c65e7ed0a55bec5246fd79e37d57b4956a00))
* refactor tests for better consistency, more accurate asserts ([#218](https://github.com/momentohq/client-sdk-rust/issues/218)) ([a041485](https://github.com/momentohq/client-sdk-rust/commit/a04148528b3681112203caae0b5c9c3f2d8ba0e2))
* remove `with_` prefixes from request buidlers ([#216](https://github.com/momentohq/client-sdk-rust/issues/216)) ([28dc57a](https://github.com/momentohq/client-sdk-rust/commit/28dc57af2a1d06e83c8d5166455a5cb83724bbec))
* remove code from mod.rs and lib.rs files ([#232](https://github.com/momentohq/client-sdk-rust/issues/232)) ([7fe44b1](https://github.com/momentohq/client-sdk-rust/commit/7fe44b1f2a916230e7e045561dd442d4e36021de))
* run flush cache test separately from other tests ([#246](https://github.com/momentohq/client-sdk-rust/issues/246)) ([3d89626](https://github.com/momentohq/client-sdk-rust/commit/3d896267a3f9cef8ad3190f4ffce85559cfb989a))
* update client protos dependency ([#229](https://github.com/momentohq/client-sdk-rust/issues/229)) ([3420560](https://github.com/momentohq/client-sdk-rust/commit/342056050153ec9f597291eebfc2b96318f50e27))

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
