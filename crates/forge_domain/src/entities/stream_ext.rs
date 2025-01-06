use futures::Stream;
use futures::StreamExt as _;

/// The result of processing a stream item with internal state
pub enum Collect {
    Ready,
    Continue,
}

pub trait StreamExt: Stream + Sized {
    fn scan_stream<State, F, G>(
        self,
        mut state: State,
        f: F,
        g: G,
    ) -> impl Stream<Item = Self::Item>
    where
        State: Clone,

        F: Fn(&mut State, &Self::Item) -> Collect + 'static,
        G: Fn(&State) -> Self::Item + 'static,
    {
        self.flat_map(move |item| match f(&mut state, &item) {
            Collect::Ready => tokio_stream::iter(vec![item, g(&state)]),
            Collect::Continue => tokio_stream::iter(vec![item]),
        })
    }
}

impl<S: Stream> StreamExt for S {}

#[cfg(test)]
mod tests {
    use futures::{stream, StreamExt};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::entities::stream_ext::StreamExt as _;

    #[tokio::test]
    async fn test_scan_stream_continue() {
        let input = vec![1, 2, 3];

        let result = stream::iter(input.clone())
            .scan_stream(
                0,
                |state, &item| {
                    *state += item;
                    Collect::Continue
                },
                |&state| state,
            )
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result, input);
    }

    #[tokio::test]
    async fn test_scan_stream_ready() {
        let input = vec![1, 2, 3];
        let count = 0;

        let result = stream::iter(input)
            .scan_stream(
                count,
                |state, _| {
                    *state += 1;
                    if *state >= 2 {
                        Collect::Ready
                    } else {
                        Collect::Continue
                    }
                },
                |&state| state * 10,
            )
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result, vec![1, 2, 20, 3, 30]);
    }

    #[tokio::test]
    async fn test_scan_stream_state_management() {
        let input = vec![1, 2, 3];
        let initial_sum = 0;

        let result = stream::iter(input)
            .scan_stream(
                initial_sum,
                |state, &item| {
                    *state += item;
                    Collect::Ready
                },
                |&state| state,
            )
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result, vec![1, 1, 2, 3, 3, 6]);
    }

    #[tokio::test]
    async fn test_scan_stream_empty() {
        let input: Vec<i32> = vec![];
        let state = 0;

        let result = stream::iter(input)
            .scan_stream(
                state,
                |state, &item| {
                    *state += item;
                    Collect::Continue
                },
                |&state| state,
            )
            .collect::<Vec<_>>()
            .await;

        assert_eq!(result, Vec::<i32>::new());
    }
}
