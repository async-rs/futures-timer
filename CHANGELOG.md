## 2018-02-07, Version 0.1.1
### Commits
- [[`4637e672f8`](https://github.com/alexcrichton/futures-timer/commit/4637e672f8748c6ee41ebc93a1a97e118ef8b855)] Bump to 0.1.1 (Alex Crichton)
- [[`0cf3dab722`](https://github.com/alexcrichton/futures-timer/commit/0cf3dab722860b5623cc835d10a1f78d953c63d9)] Add extension traits for convenience methods (Alex Crichton)
- [[`56c46fe881`](https://github.com/alexcrichton/futures-timer/commit/56c46fe8812cde701dd983d843c8f92189b2f911)] Beef up the README slightly (Alex Crichton)
- [[`dfd80e2e2b`](https://github.com/alexcrichton/futures-timer/commit/dfd80e2e2b011bc0a790d88f8219fc838e02b608)] Various updates: (Alex Crichton)
- [[`2f8e18cc71`](https://github.com/alexcrichton/futures-timer/commit/2f8e18cc718a3bc57cabcbbeaf79cbead0bb47f0)] Rename `Sleep` to `Delay` (Alex Crichton)
- [[`9715aa6417`](https://github.com/alexcrichton/futures-timer/commit/9715aa64176ef006a4bcda05dc8188160476a2d6)] Clarify wording of license information in README. (Alex Crichton)
- [[`38e14656d7`](https://github.com/alexcrichton/futures-timer/commit/38e14656d76cb9478eaea0c54b1de117bacd63ee)] Rename Timeout to Sleep (Alex Crichton)
- [[`9e58f3a330`](https://github.com/alexcrichton/futures-timer/commit/9e58f3a3307422042b93787cb18d93cf6c6d6671)] Import some tokio-core tests (Alex Crichton)
- [[`9c4a3958cd`](https://github.com/alexcrichton/futures-timer/commit/9c4a3958cde60156941415bb806db05d21ca3f1b)] Correct take_and_seal implementation (Alex Crichton)
- [[`56dddd9d57`](https://github.com/alexcrichton/futures-timer/commit/56dddd9d5716ed6cef8841f70c3be6d1536f2dac)] Fix some races with invalidating timeouts (Alex Crichton)
- [[`1653e6c789`](https://github.com/alexcrichton/futures-timer/commit/1653e6c7896348ec928c71dd272cecd8b26ec951)] typo (Alex Crichton)
- [[`e18b12069b`](https://github.com/alexcrichton/futures-timer/commit/e18b12069b50a02a217aa86945e5b68ac89e1459)] Add a doc link (Alex Crichton)
- [[`c3d239e874`](https://github.com/alexcrichton/futures-timer/commit/c3d239e8740a217a3a352ad3538e4bd694794097)] Add various metadata (Alex Crichton)
- [[`57324b99d4`](https://github.com/alexcrichton/futures-timer/commit/57324b99d4b2906bda548514b2bf238c7f83aee6)] Add Travis config (Alex Crichton)
- [[`4acd39db09`](https://github.com/alexcrichton/futures-timer/commit/4acd39db096d97c5f9d60380194b9680a402ebf7)] Make `fires_at` private (Alex Crichton)
- [[`7b3f3b05f0`](https://github.com/alexcrichton/futures-timer/commit/7b3f3b05f0c2edc8f502fb22f406dc9c11d55da8)] Add Interval (Alex Crichton)
- [[`6b65dd457a`](https://github.com/alexcrichton/futures-timer/commit/6b65dd457a9d8f1e9b78cb3d0e57e74ed8bb10be)] Fix reset (Alex Crichton)
- [[`2603432e78`](https://github.com/alexcrichton/futures-timer/commit/2603432e78322c4f272509b3e938d7a58194d023)] Initial commit (Alex Crichton)

### Stats
```diff
 .travis.yml       |   4 +-
 CHANGELOG.md      |  40 +----------------
 Cargo.toml        |   9 +----
 README.md         |  29 +++++++-----
 src/arc_list.rs   |  16 +++----
 src/delay.rs      |  72 +++++++++--------------------
 src/ext.rs        | 135 ++++++++++++++++++-------------------------------------
 src/global.rs     |  52 ++++++---------------
 src/heap.rs       |  56 +++++++++--------------
 src/interval.rs   |  87 +++++++++++++----------------------
 src/lib.rs        |  90 ++++++++++++++++---------------------
 tests/interval.rs |  22 ++++-----
 tests/smoke.rs    |  64 ++++++++++----------------
 tests/timeout.rs  |  26 +++++------
 14 files changed, 261 insertions(+), 441 deletions(-)
```


## 2018-02-07, Version 0.1.1
### Commits
- [[`4637e672f8`](https://github.com/alexcrichton/futures-timer/commit/4637e672f8748c6ee41ebc93a1a97e118ef8b855)] Bump to 0.1.1 (Alex Crichton)
- [[`0cf3dab722`](https://github.com/alexcrichton/futures-timer/commit/0cf3dab722860b5623cc835d10a1f78d953c63d9)] Add extension traits for convenience methods (Alex Crichton)
- [[`56c46fe881`](https://github.com/alexcrichton/futures-timer/commit/56c46fe8812cde701dd983d843c8f92189b2f911)] Beef up the README slightly (Alex Crichton)
- [[`dfd80e2e2b`](https://github.com/alexcrichton/futures-timer/commit/dfd80e2e2b011bc0a790d88f8219fc838e02b608)] Various updates: (Alex Crichton)
- [[`2f8e18cc71`](https://github.com/alexcrichton/futures-timer/commit/2f8e18cc718a3bc57cabcbbeaf79cbead0bb47f0)] Rename `Sleep` to `Delay` (Alex Crichton)
- [[`9715aa6417`](https://github.com/alexcrichton/futures-timer/commit/9715aa64176ef006a4bcda05dc8188160476a2d6)] Clarify wording of license information in README. (Alex Crichton)
- [[`38e14656d7`](https://github.com/alexcrichton/futures-timer/commit/38e14656d76cb9478eaea0c54b1de117bacd63ee)] Rename Timeout to Sleep (Alex Crichton)
- [[`9e58f3a330`](https://github.com/alexcrichton/futures-timer/commit/9e58f3a3307422042b93787cb18d93cf6c6d6671)] Import some tokio-core tests (Alex Crichton)
- [[`9c4a3958cd`](https://github.com/alexcrichton/futures-timer/commit/9c4a3958cde60156941415bb806db05d21ca3f1b)] Correct take_and_seal implementation (Alex Crichton)
- [[`56dddd9d57`](https://github.com/alexcrichton/futures-timer/commit/56dddd9d5716ed6cef8841f70c3be6d1536f2dac)] Fix some races with invalidating timeouts (Alex Crichton)
- [[`1653e6c789`](https://github.com/alexcrichton/futures-timer/commit/1653e6c7896348ec928c71dd272cecd8b26ec951)] typo (Alex Crichton)
- [[`e18b12069b`](https://github.com/alexcrichton/futures-timer/commit/e18b12069b50a02a217aa86945e5b68ac89e1459)] Add a doc link (Alex Crichton)
- [[`c3d239e874`](https://github.com/alexcrichton/futures-timer/commit/c3d239e8740a217a3a352ad3538e4bd694794097)] Add various metadata (Alex Crichton)
- [[`57324b99d4`](https://github.com/alexcrichton/futures-timer/commit/57324b99d4b2906bda548514b2bf238c7f83aee6)] Add Travis config (Alex Crichton)
- [[`4acd39db09`](https://github.com/alexcrichton/futures-timer/commit/4acd39db096d97c5f9d60380194b9680a402ebf7)] Make `fires_at` private (Alex Crichton)
- [[`7b3f3b05f0`](https://github.com/alexcrichton/futures-timer/commit/7b3f3b05f0c2edc8f502fb22f406dc9c11d55da8)] Add Interval (Alex Crichton)
- [[`6b65dd457a`](https://github.com/alexcrichton/futures-timer/commit/6b65dd457a9d8f1e9b78cb3d0e57e74ed8bb10be)] Fix reset (Alex Crichton)
- [[`2603432e78`](https://github.com/alexcrichton/futures-timer/commit/2603432e78322c4f272509b3e938d7a58194d023)] Initial commit (Alex Crichton)

### Stats
```diff
 .travis.yml       |   4 +-
 Cargo.toml        |   9 +----
 README.md         |  27 +++++++----
 src/arc_list.rs   |  16 +++----
 src/delay.rs      |  65 +++++++++------------------
 src/ext.rs        | 133 +++++++++++++++++++------------------------------------
 src/global.rs     |  52 ++++++----------------
 src/heap.rs       |  56 +++++++++--------------
 src/interval.rs   |  86 ++++++++++++++----------------------
 src/lib.rs        |  76 +++++++++++++++----------------
 tests/interval.rs |  22 ++++-----
 tests/smoke.rs    |  64 ++++++++++----------------
 tests/timeout.rs  |  26 +++++------
 13 files changed, 260 insertions(+), 376 deletions(-)
```


