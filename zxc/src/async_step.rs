use std::fmt::Display;

use tracing::{Instrument, Level, span};

// Trait like AsyncIterator. Used in async_run().
pub trait AsyncStep {
    type Error;

    async fn next(self) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn is_ended(&self) -> bool;
}

/* Description:
 *      Function with AsyncStep trait bound. Continues calling next() until
 *      is_ended() returns true.
 */

pub async fn async_run<T>(mut state: T) -> Result<T, T::Error>
where
    T: AsyncStep + Display,
{
    loop {
        let span = span(&state);
        state = state.next().instrument(span).await?;
        if state.is_ended() {
            return Ok(state);
        }
    }
}

fn span<T>(state: T) -> tracing::Span
where
    T: Display,
{
    let span = span!(Level::TRACE, "", "{}", state.to_string());
    let _ = span.enter();
    span
}
