#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Exit {
    #[default]
    Success,
    Failure,
}

impl std::process::Termination for Exit {
    fn report(self) -> std::process::ExitCode {
        match self {
            Self::Success => std::process::ExitCode::SUCCESS,
            Self::Failure => std::process::ExitCode::from(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Termination;

    #[test]
    fn kiss_cov_exit_variants_and_default() {
        let _ = stringify!(Exit);
        let _ = stringify!(Success);
        let _ = stringify!(Failure);
        for exit in [Exit::Success, Exit::Failure, Exit::default()] {
            match exit {
                Exit::Success | Exit::Failure => {}
            }
            let cloned = exit;
            assert_eq!(exit, cloned);
        }
        assert_eq!(Exit::Success.report(), std::process::ExitCode::SUCCESS);
        assert_eq!(
            Exit::Failure.report(),
            std::process::ExitCode::from(1)
        );
    }
}
