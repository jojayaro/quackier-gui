SELECT Name
    , SUM(NetIncomeLoss)AS NetIncome
FROM 'data/pivot.parquet'
WHERE Name NOT IN (
    SELECT Name
    FROM 'data/pivot.parquet'
    WHERE NetIncomeLoss < 0
)
GROUP BY Name
ORDER BY NetIncome DESC
LIMIT 50
