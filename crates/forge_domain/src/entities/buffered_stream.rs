use futures::{Stream, StreamExt};

/// The result of processing a stream item with internal state
pub enum Collect {
    Ready,
    Continue,
}

pub fn scan_stream<Source, State, I, F, G>(
    s: Source,
    mut state: State,
    f: F,
    g: G,
) -> impl Stream<Item = I>
where
    State: Clone,
    Source: Stream<Item = I>,
    F: Fn(&mut State, &I) -> Collect + 'static,
    G: Fn(&State) -> I + 'static,
{
    s.flat_map(move |item| match f(&mut state, &item) {
        Collect::Ready => tokio_stream::iter(vec![item, g(&state)]),
        Collect::Continue => tokio_stream::iter(vec![item]),
    })
}

#[cfg(test)]
mod tests {
    use futures::stream;
    use pretty_assertions::assert_eq;
    use tokio_stream::StreamExt;

    use super::*;

    #[tokio::test]
    async fn test_scan_stream_continue() {
        let input = vec![1, 2, 3];

        let result = scan_stream(
            stream::iter(input.clone()),
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

        let result = scan_stream(
            stream::iter(input),
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

        let result = scan_stream(
            stream::iter(input),
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

        let result = scan_stream(
            stream::iter(input),
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
