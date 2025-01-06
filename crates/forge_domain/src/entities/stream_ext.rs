use futures::{Stream, StreamExt as _};

pub trait StreamExt {
    fn try_collect<State, F, A, E>(self, mut state: State, f: F) -> impl Stream<Item = Result<A, E>>
    where
        State: Clone,
        Self: Stream<Item = Result<A, E>> + Sized,
        F: Fn(&mut State, &A) -> Result<Option<A>, E> + 'static,
    {
        self.flat_map(move |item| match item {
            Ok(item) => match f(&mut state, &item) {
                Ok(Some(new_item)) => tokio_stream::iter(vec![Ok(item), Ok(new_item)]),
                Ok(None) => tokio_stream::iter(vec![Ok(item)]),
                Err(err) => tokio_stream::iter(vec![Err(err)]),
            },
            Err(err) => tokio_stream::iter(vec![Err(err)]),
        })
    }
}

impl<S: Stream> StreamExt for S {}

#[cfg(test)]
mod tests {
    use futures::{stream, StreamExt as _};
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestError;

    #[tokio::test]
    async fn test_try_collect_pass_through() {
        let input = vec![1, 2, 3]
            .into_iter()
            .map(Ok::<i32, TestError>)
            .collect::<Vec<_>>();

        let result = stream::iter(input.clone())
            .try_collect(0, |state, item| {
                *state += item;
                Ok::<Option<i32>, TestError>(None)
            })
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result, input);
    }

    #[tokio::test]
    async fn test_try_collect_with_new_items() {
        let input = vec![1, 2, 3]
            .into_iter()
            .map(Ok::<i32, TestError>)
            .collect::<Vec<_>>();

        let result = stream::iter(input)
            .try_collect(0, |state, item| {
                *state += item;

                Ok(Some(*state))
            })
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result, vec![Ok(1), Ok(1), Ok(2), Ok(3), Ok(3), Ok(6)]);
    }

    #[tokio::test]
    async fn test_try_collect_empty() {
        let input: Vec<Result<i32, TestError>> = vec![];

        let result = stream::iter(input)
            .try_collect(0, |_state, _item| Ok::<Option<i32>, TestError>(None))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result, Vec::<Result<i32, TestError>>::new());
    }

    #[tokio::test]
    async fn test_try_collect_error_handling() {
        let input = vec![Ok(1), Ok(2), Err(TestError), Ok(3)];

        let result = stream::iter(input.clone())
            .try_collect(0, |state, item| {
                *state += item;
                Ok::<Option<i32>, TestError>(None)
            })
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result, input);
    }
}
