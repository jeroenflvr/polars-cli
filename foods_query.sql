SELECT
    category,
    calories
FROM read_csv('foods_semicolon.csv', separator = ';')
WHERE calories > 100;
