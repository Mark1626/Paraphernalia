-- A small synthetic workload: overlapping filters, a repeated join, a repeated
-- aggregate, one near-duplicate pair, one fragment/superset pair, one bad query.

SELECT o.id, o.total
FROM orders o
WHERE o.status = 'shipped' AND o.total > 100;

SELECT o.id, o.total
FROM orders o
WHERE o.total > 250 AND o.status = 'pending';

SELECT c.name, SUM(o.total) AS revenue
FROM orders o
JOIN customers c ON o.customer_id = c.id
WHERE o.status = 'shipped' AND o.total > 100
GROUP BY c.name
ORDER BY revenue DESC
LIMIT 10;

SELECT c.name, SUM(o.total) AS revenue
FROM orders AS o
JOIN customers AS c ON c.id = o.customer_id
WHERE o.total > 100 AND o.status = 'shipped'
GROUP BY c.name
ORDER BY revenue DESC
LIMIT 10;

SELECT c.name, COUNT(*) AS n_orders, SUM(o.total) AS revenue
FROM orders o
JOIN customers c ON o.customer_id = c.id
WHERE o.status = 'shipped' AND o.total > 100 AND c.region = 'EU'
GROUP BY c.name;

WITH recent AS (
  SELECT o.id, o.customer_id, o.total
  FROM orders o
  WHERE o.status = 'shipped' AND o.total > 100
)
SELECT c.name, SUM(r.total) AS revenue
FROM recent r
JOIN customers c ON r.customer_id = c.id
GROUP BY c.name;

SELECT l.sku, SUM(l.qty) AS units
FROM order_lines l
JOIN orders o ON l.order_id = o.id
WHERE o.status = 'shipped'
GROUP BY l.sku
HAVING SUM(l.qty) > 1000;

SELECT sku, SUM(qty) AS units
FROM order_lines
WHERE order_id IN (1, 2, 3, 4, 5)
GROUP BY sku;

SELECT p.name, p.price
FROM products p
WHERE p.price BETWEEN 10 AND 20 AND p.discontinued = false;

SELECT COUNT(*) FROM customers WHERE region = 'EU';

SELEKT * FROM broken WHERE;
