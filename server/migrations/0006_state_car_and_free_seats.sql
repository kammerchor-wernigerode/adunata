ALTER TABLE responses
    ADD COLUMN car BOOLEAN,
    ADD COLUMN free_seats SMALLINT CHECK (free_seats BETWEEN 0 AND 8),
    ADD CONSTRAINT responses_no_car_when_declined CHECK (
        status <> 'declined' OR (car IS NULL AND free_seats IS NULL)
    ),
    ADD CONSTRAINT responses_free_seats_with_car CHECK (
        (car IS TRUE) = (free_seats IS NOT NULL)
    );
