
FORMAT: 1A

# coinched

The coinched API allows to communicate with a coinched server to play the coinche card game.

Desired properties:

* Calling twice the same page should not cause error
* Ideally, calling twice the same web page should return the same thing

Player refinement:

* Anonymous players, UUID generated on /join
* Named player, name is chosen on /join, no registration
* Registrated players, use password?

# Group Public
These methods can be called without a player ID.

## GET /help
Returns an help message with the available API endpoints.

+ Response 404 (applicatiion/json)

## POST /join
Attempt to join a new game. Will block until a party is found.

+ Response 200 (application/json)

        {
          "player_id": 123456,
          "player_pos": 2
        }
