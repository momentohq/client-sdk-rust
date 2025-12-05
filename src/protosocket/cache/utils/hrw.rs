use std::sync::Arc;
use std::vec::IntoIter;

use xxhash_rust::xxh3::xxh3_64;

pub fn hrw_hash(to_hash: &[u8]) -> i32 {
    xxh3_64(to_hash) as i32
}

/// seed/partitionPlacementSeed/partition address + variadic factors based
/// on algorithm mentioned in http://www.eecs.umich.edu/techreports/cse/96/CSE-TR-316-96.pdf
/// which is based in turn on the original BSD rand() function.
fn weight(placement_seed: i32, item_digest: i32) -> i32 {
    let rand: i32 = 1103515245;
    rand.wrapping_mul(rand.wrapping_mul(placement_seed) ^ item_digest)
}

pub(crate) trait PlacementTarget {
    fn placement_seed(&self) -> i32;
    fn weigh(&self, item_digest: i32) -> i32 {
        weight(self.placement_seed(), item_digest)
    }
}

impl<T> PlacementTarget for Arc<T>
where
    T: PlacementTarget,
{
    fn placement_seed(&self) -> i32 {
        (**self).placement_seed()
    }
}

pub(crate) fn place_targets<T: PlacementTarget>(
    key: &[u8],
    placement_factor_digest: i32,
    targets: impl Iterator<Item = T>,
) -> IntoIter<T> {
    let key_factor = placement_factor_digest ^ hrw_hash(key);
    let mut targets_vec: Vec<T> = targets.collect();
    targets_vec.sort_by(|a, b| Ord::cmp(&a.weigh(key_factor), &b.weigh(key_factor)));
    targets_vec.into_iter()
}

#[cfg(test)]
mod test {
    use super::{hrw_hash, place_targets, weight, PlacementTarget};

    struct TestPlacementTarget {
        placement_seed: i32,
    }

    impl TestPlacementTarget {
        fn new(placement_seed: i32) -> Self {
            Self { placement_seed }
        }
    }

    impl PlacementTarget for &TestPlacementTarget {
        fn placement_seed(&self) -> i32 {
            self.placement_seed
        }
    }

    struct TestPlacementTargetCompoundFactors {
        placement_seed: i32,
        fixed_digest_factors: Vec<Vec<u8>>,
    }

    impl TestPlacementTargetCompoundFactors {
        fn new(placement_seed: i32, fixed_digest_factors: Vec<Vec<u8>>) -> Self {
            Self {
                placement_seed,
                fixed_digest_factors,
            }
        }

        fn placement_factor_digest(&self) -> i32 {
            self.fixed_digest_factors
                .iter()
                .map(|to_hash| hrw_hash(to_hash))
                .fold(0, |acc, hash| acc ^ hash)
        }
    }

    impl PlacementTarget for &TestPlacementTargetCompoundFactors {
        fn placement_seed(&self) -> i32 {
            self.placement_seed
        }
        fn weigh(&self, variable_digest_factor: i32) -> i32 {
            weight(
                self.placement_seed,
                self.placement_factor_digest() ^ variable_digest_factor,
            )
        }
    }

    #[test]
    fn hrw_hash_produces_expected_value() {
        let to_hash_list = [
            "yaxjtq74i",
            "jewmilumw",
            "eza324oan",
            "u7f3gojrb",
            "rg-6pb2uwqxo",
            "rg-kj4mygscp",
        ];
        let expected_hash_values = vec![
            1499626933, 567767100, 1362330692, -883158939, 581806593, 2033962128,
        ];
        let result = to_hash_list
            .iter()
            .map(|s| hrw_hash(s.as_bytes()))
            .collect::<Vec<_>>();
        assert_eq!(expected_hash_values, result);
    }

    #[test]
    fn place_targets_returns_expected_winner() {
        let placement_seeds = [-1080413201, -648207022, 5038413];
        let targets = [
            TestPlacementTarget::new(placement_seeds[0]),
            TestPlacementTarget::new(placement_seeds[1]),
            TestPlacementTarget::new(placement_seeds[2]),
        ];
        let expected_winners = [1, 0, 1, 2, 2, 1, 1, 2, 0, 0, 0, 0, 2, 1, 0, 0, 0, 2, 0, 2]
            .iter()
            .map(|i| placement_seeds[*i])
            .collect::<Vec<_>>();
        let winners = (1..=20)
            .map(|i| {
                place_targets(format!("MyKey{i}").as_bytes(), 2039064864, targets.iter())
                    .next()
                    .expect("should get hrw winner")
                    .placement_seed
            })
            .collect::<Vec<_>>();
        assert_eq!(expected_winners, winners);
    }

    const KEY: &str = "here is a cache key, it has some length.";

    fn targets() -> Vec<TestPlacementTargetCompoundFactors> {
        (1..=9)
            .map(|i| {
                TestPlacementTargetCompoundFactors::new(
                    i,
                    vec![
                        format!("factor{i}1").into_bytes(),
                        format!("factor{}2", i + 10).into_bytes(),
                    ],
                )
            })
            .collect()
    }

    fn expected_target_seed_order() -> Vec<i32> {
        vec![8, 7, 4, 9, 5, 1, 3, 6, 2]
    }

    #[test]
    fn hrw_is_stable() {
        for _ in 0..10 {
            let sorted_target_seeds = place_targets(KEY.as_bytes(), 0, targets().iter())
                .map(|target| target.placement_seed)
                .collect::<Vec<_>>();
            assert_eq!(expected_target_seed_order(), sorted_target_seeds);
        }
    }

    #[test]
    fn hrw_keeps_the_same_ordering() {
        let mut targets = targets();

        // removing one target
        let sorted_target_seeds = place_targets(
            KEY.as_bytes(),
            0,
            targets.iter().filter(|target| target.placement_seed != 3),
        )
        .map(|target| target.placement_seed)
        .collect::<Vec<_>>();
        assert_eq!(
            expected_target_seed_order()
                .into_iter()
                .filter(|s| *s != 3)
                .collect::<Vec<_>>(),
            sorted_target_seeds
        );

        // adding one target
        let a_target =
            TestPlacementTargetCompoundFactors::new(42, vec!["some seed".as_bytes().to_vec()]);
        targets.push(a_target);
        let sorted_target_seeds =
            place_targets(KEY.as_bytes(), 0, targets.iter()).map(|target| target.placement_seed);
        assert_eq!(
            expected_target_seed_order(),
            sorted_target_seeds.filter(|s| *s != 42).collect::<Vec<_>>()
        );
    }

    #[test]
    fn emtpy_targets() {
        let targets: Vec<TestPlacementTarget> = Vec::new();
        assert!(place_targets(KEY.as_bytes(), 0, targets.iter())
            .next()
            .is_none());
    }
}
