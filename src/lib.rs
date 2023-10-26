use std::{future::Future, time::Duration};

use zduny_wasm_timer::Delay;

#[derive(Clone, Copy)]
pub struct BackoffSettings {
    pub initial_backoff: Duration,
    pub total_retries: usize,
}

impl Default for BackoffSettings {
    fn default() -> Self {
        Self {
            initial_backoff: Duration::from_millis(50),
            total_retries: 5,
        }
    }
}

pub async fn retry<F, Fut, T, E>(op: F, settings: BackoffSettings) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: From<std::io::Error>,
{
    let mut counter = 0;
    let mut backoff = settings.initial_backoff;

    loop {
        let res = op().await;
        if res.is_ok() || counter >= settings.total_retries {
            break res;
        }

        Delay::new(backoff).await?;
        counter += 1;
        backoff *= 2;
    }
}

#[cfg(test)]
mod tests {
    use crate::{retry, BackoffSettings};

    #[tokio::test]
    async fn test_retry() {
        retry(
            || async { Ok::<_, eyre::Report>(5) },
            BackoffSettings::default(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_retry_fails() {
        let res = retry(
            || async { Err::<usize, _>(eyre::eyre!("error")) },
            BackoffSettings::default(),
        )
        .await;

        assert!(res.is_err());
    }
}
