SELECT * FROM housemate.users_by_username
WHERE lastname LIKE ?
USING TIMEOUT 10000ms