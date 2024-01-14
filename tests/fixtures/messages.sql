INSERT INTO
    messages (
        id,
        message_sid,
        origin,
        destination,
        body,
        initial_message_id,
        created_at,
        updated_at
    )
VALUES
    (
        '8a8c40e1-e7b6-497b-9cca-550665f48922',
        'SM001',
        'approved',
        'textabus',
        'hello',
        NULL,
        '2019-01-01 00:00:00',
        '2019-01-01 00:00:00'
    ),
    (
        '4addcf7f-dd8a-4cd8-94ca-e37c7d9f6519',
        NULL,
        'textabus',
        'approved',
        'hey',
        '8a8c40e1-e7b6-497b-9cca-550665f48922',
        '2019-01-01 00:00:01',
        '2019-01-01 00:00:01'
    ),
    (
        'b3bea420-3783-4abc-9f66-ed9007e698e8',
        NULL,
        'textabus',
        'approved',
        '?',
        '8a8c40e1-e7b6-497b-9cca-550665f48922',
        '2019-01-01 00:00:02',
        '2019-01-01 00:00:02'
    ),
    (
        'b206e675-9220-4d95-94a8-a3dc0737557b',
        'SM002',
        'stranger',
        'textabus',
        'hello',
        NULL,
        NOW(),
        NOW()
    ),
    (
        '618e7375-932e-4a11-b472-e608d4e28139',
        NULL,
        'textabus',
        'stranger',
        'who are you',
        'b206e675-9220-4d95-94a8-a3dc0737557b',
        NOW(),
        NOW()
    );