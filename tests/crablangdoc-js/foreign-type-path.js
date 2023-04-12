const QUERY = 'MyForeignType::my_method';

const EXPECTED = {
    'others': [
        // Test case for https://github.com/crablang/crablang/pull/96887#pullrequestreview-967154358
        // Validates that the parent path for a foreign type method is correct.
        { 'path': 'foreign_type_path::aaaaaaa::MyForeignType', 'name': 'my_method' },
    ],
};
