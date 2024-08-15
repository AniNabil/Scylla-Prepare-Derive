SELECT * FROM housemate.users_by_username
WHERE lastname LIKE ?
LIMIT 5
USING TIMEOUT 500ms