// ignore-order

const QUERY = 'RawFd::as_raw_fd';

const EXPECTED = {
    'others': [
        // Reproduction test for https://github.com/crablang/crablang/issues/78724
        // Validate that type alias methods get the correct path.
        { 'path': 'std::os::fd::AsRawFd', 'name': 'as_raw_fd' },
        { 'path': 'std::os::fd::AsRawFd', 'name': 'as_raw_fd' },
        { 'path': 'std::os::linux::process::PidFd', 'name': 'as_raw_fd' },
        { 'path': 'std::os::fd::RawFd', 'name': 'as_raw_fd' },
    ],
};
