use momento::leaderboard::{Order, ScoreRange};
use momento::{Leaderboard, MomentoResult};
use momento_test_util::{unique_leaderboard_name, TestLeaderboard, CACHE_TEST_STATE};

fn unique_leaderboard() -> Leaderboard {
    let client = &CACHE_TEST_STATE.leaderboard_client;
    let cache_name = CACHE_TEST_STATE.cache_name.as_str();
    let leaderboard_name = unique_leaderboard_name();
    client.leaderboard(cache_name, leaderboard_name)
}

mod upsert {
    use super::*;

    #[tokio::test]
    async fn upsert_elements() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_data = TestLeaderboard::new();
        leaderboard.upsert(test_data.elements()).await?;

        let response = leaderboard.fetch_by_score(ScoreRange::unbounded()).await?;
        assert_eq!(
            test_data.ranked_elements(),
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }
}

mod length {
    use super::*;

    #[tokio::test]
    async fn length_of_existing_leaderboard() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        leaderboard.upsert(vec![(123, 1.0), (456, 2.0)]).await?;
        let response = leaderboard.len().await?;

        assert_eq!(
            2,
            response.length(),
            "Expected two elements in the leaderboard"
        );
        Ok(())
    }

    #[tokio::test]
    async fn length_of_non_existent_leaderboard() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let response = leaderboard.len().await?;

        assert_eq!(
            0,
            response.length(),
            "Expected zero elements in the leaderboard"
        );
        Ok(())
    }
}

mod delete {
    use super::*;

    #[tokio::test]
    async fn delete_existing_leaderboard() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        leaderboard.upsert(vec![(123, 1.0), (456, 2.0)]).await?;
        let length_response = leaderboard.len().await?;
        assert_eq!(
            2,
            length_response.length(),
            "Expected two elements in the leaderboard"
        );

        leaderboard.delete().await?;
        let length_response2 = leaderboard.len().await?;

        assert_eq!(
            0,
            length_response2.length(),
            "Expected zero elements in the leaderboard"
        );
        Ok(())
    }

    #[tokio::test]
    async fn delete_non_existent_leaderboard() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();

        leaderboard.delete().await?;

        let response = leaderboard.len().await?;
        assert_eq!(
            0,
            response.length(),
            "Expected zero elements in the leaderboard"
        );
        Ok(())
    }
}

mod remove_elements {
    use super::*;

    #[tokio::test]
    async fn remove_an_element() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();

        leaderboard.upsert(test_leaderboard.elements()).await?;
        let length_response = leaderboard.len().await?;
        assert_eq!(
            2,
            length_response.length(),
            "Expected two elements in the leaderboard"
        );

        leaderboard
            .remove_elements(vec![test_leaderboard.elements()[0].id])
            .await?;
        let leaderboard_response = leaderboard.fetch_by_score(ScoreRange::unbounded()).await?;

        let mut second_element = test_leaderboard.ranked_elements()[1].clone();
        second_element.rank = 0;
        assert_eq!(
            vec![second_element],
            leaderboard_response.elements(),
            "Expected one element in the leaderboard"
        );
        Ok(())
    }
}

mod get_rank {
    use momento::leaderboard::GetRankRequest;

    use super::*;

    #[tokio::test]
    async fn get_rank_of_elements() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard.get_rank(test_leaderboard.ids()).await?;

        assert_eq!(
            test_leaderboard.ranked_elements(),
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }

    #[tokio::test]
    async fn get_rank_of_elements_descending() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard
            .send_request(GetRankRequest::new(test_leaderboard.ids()).order(Order::Descending))
            .await?;

        let mut ranked_elements = test_leaderboard.ranked_elements();
        let num_elements = ranked_elements.len();
        // adjust ranks
        for (i, e) in ranked_elements.iter_mut().enumerate() {
            e.rank = (num_elements - i - 1) as u32;
        }

        assert_eq!(
            ranked_elements,
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }
}

mod fetch_by_rank {
    use momento::leaderboard::FetchByRankRequest;

    use super::*;

    #[tokio::test]
    async fn fetch_by_rank() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard.fetch_by_rank(0..2).await?;

        assert_eq!(
            test_leaderboard.ranked_elements(),
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }

    #[tokio::test]
    async fn fetch_by_rank_first() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard.fetch_by_rank(0..1).await?;

        assert_eq!(
            vec![test_leaderboard.ranked_elements()[0].clone()],
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }

    #[tokio::test]
    async fn fetch_by_rank_descending() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard
            .send_request(FetchByRankRequest::new(0..2).order(Order::Descending))
            .await?;

        let mut ranked_elements = test_leaderboard.ranked_elements();
        ranked_elements.reverse();
        // adjust ranks
        for (i, e) in ranked_elements.iter_mut().enumerate() {
            e.rank = i as u32;
        }

        assert_eq!(
            ranked_elements,
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }
}

mod fetch_by_score {
    use super::*;
    use momento::leaderboard::FetchByScoreRequest;

    #[tokio::test]
    async fn fetch_by_score() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard.fetch_by_score(ScoreRange::unbounded()).await?;

        assert_eq!(
            test_leaderboard.ranked_elements(),
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }

    #[tokio::test]
    async fn fetch_by_score_descending() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard
            .send_request(
                FetchByScoreRequest::new(-f64::INFINITY..f64::INFINITY).order(Order::Descending),
            )
            .await?;

        let mut ranked_elements = test_leaderboard.ranked_elements();
        ranked_elements.reverse();
        // adjust ranks
        for (i, e) in ranked_elements.iter_mut().enumerate() {
            e.rank = i as u32;
        }

        assert_eq!(
            ranked_elements,
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }

    #[tokio::test]
    async fn fetch_by_score_with_min_max() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard.fetch_by_score(1.0..2.0).await?;

        assert_eq!(
            vec![test_leaderboard.ranked_elements()[0].clone()],
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }

    #[tokio::test]
    async fn fetch_by_score_with_min_max_descending() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard
            .send_request(FetchByScoreRequest::new(1.5..3.0).order(Order::Descending))
            .await?;

        let mut ranked_element = test_leaderboard.ranked_elements()[1].clone();
        ranked_element.rank = 0;

        assert_eq!(
            vec![ranked_element],
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }

    #[tokio::test]
    async fn fetch_by_score_with_offset_and_count() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        let test_leaderboard = TestLeaderboard::new();
        leaderboard.upsert(test_leaderboard.elements()).await?;

        let response = leaderboard
            .send_request(
                FetchByScoreRequest::new(ScoreRange::unbounded())
                    .offset(1)
                    .count(1),
            )
            .await?;

        let ranked_element = test_leaderboard.ranked_elements()[1].clone();

        assert_eq!(
            vec![ranked_element],
            response.elements(),
            "Expected the leaderboard to contain the elements that were upserted"
        );
        Ok(())
    }
}

mod get_competition_rank {
    use momento::leaderboard::{
        messages::data::get_competition_rank::GetCompetitionRankRequest, Element, RankedElement,
    };

    use super::*;

    fn test_competition_leaderboard() -> Vec<Element> {
        vec![
            Element { id: 0, score: 20.0 },
            Element { id: 1, score: 10.0 },
            Element { id: 2, score: 10.0 },
            Element { id: 3, score: 5.0 },
        ]
    }

    #[tokio::test]
    async fn get_competition_rank_of_elements() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        leaderboard.upsert(test_competition_leaderboard()).await?;

        let response = leaderboard.get_competition_rank([0, 1, 2, 3, 4]).await?;

        assert_eq!(
            vec![
                RankedElement {
                    id: 0,
                    score: 20.0,
                    rank: 0
                },
                RankedElement {
                    id: 1,
                    score: 10.0,
                    rank: 1
                },
                RankedElement {
                    id: 2,
                    score: 10.0,
                    rank: 1
                },
                RankedElement {
                    id: 3,
                    score: 5.0,
                    rank: 3
                },
            ],
            response.elements(),
            "Expected the leaderboard to be sorted in 0113 order"
        );
        Ok(())
    }

    #[tokio::test]
    async fn get_rank_of_elements_asc() -> MomentoResult<()> {
        let leaderboard = unique_leaderboard();
        leaderboard.upsert(test_competition_leaderboard()).await?;

        let response = leaderboard
            .send_request(GetCompetitionRankRequest::new([0, 1, 2, 3, 4]).order(Order::Ascending))
            .await?;

        assert_eq!(
            vec![
                RankedElement {
                    id: 0,
                    score: 20.0,
                    rank: 3
                },
                RankedElement {
                    id: 1,
                    score: 10.0,
                    rank: 1
                },
                RankedElement {
                    id: 2,
                    score: 10.0,
                    rank: 1
                },
                RankedElement {
                    id: 3,
                    score: 5.0,
                    rank: 0
                },
            ],
            response.elements(),
            "Expected the leaderboard to be sorted in 3110 order"
        );
        Ok(())
    }
}
