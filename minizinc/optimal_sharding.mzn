% Optimal 71-Shard Distribution Model
% Proves maximum information embedding across platforms

include "globals.mzn";

% Constants
int: NUM_SHARDS = 71;  % Gandalf threshold

% Platform types
enum Platform = {
  Twitter, Discord, Telegram, GitHub_Commit, GitHub_Body,
  Website, Email, Reddit, Mastodon, Bluesky
};

% Platform limits (characters after compression)
array[Platform] of int: platform_limits = [
  180,    % Twitter (280 chars → ~180 bytes with compression)
  1500,   % Discord (2000 chars → ~1500 bytes)
  3000,   % Telegram (4096 chars → ~3000 bytes)
  50,     % GitHub commit first line
  45000,  % GitHub body (65536 chars → ~45KB)
  50000,  % Website (effectively unlimited, cap at 50KB)
  10000,  % Email
  35000,  % Reddit (40000 chars → ~35KB)
  300,    % Mastodon (500 chars → ~300 bytes)
  240     % Bluesky (300 chars → ~240 bytes)
];

% Decision variables: which platform for each shard
array[1..NUM_SHARDS] of var Platform: shard_platform;

% Decision variables: bytes per shard
array[1..NUM_SHARDS] of var int: shard_bytes;

% Constraint: shard size must fit in platform limit
constraint forall(i in 1..NUM_SHARDS) (
  shard_bytes[i] <= platform_limits[shard_platform[i]]
);

% Constraint: shard size must be positive
constraint forall(i in 1..NUM_SHARDS) (
  shard_bytes[i] >= 1
);

% Constraint: distribute shards across platforms (at least 1 per platform)
constraint forall(p in Platform) (
  count(shard_platform, p) >= 1
);

% Constraint: balance distribution (no platform gets more than 20%)
constraint forall(p in Platform) (
  count(shard_platform, p) <= 15  % ~20% of 71
);

% Constraint: critical shards (1-10) on high-capacity platforms
constraint forall(i in 1..10) (
  shard_platform[i] in {Website, GitHub_Body, Reddit}
);

% Constraint: maximize shard size for each platform
constraint forall(i in 1..NUM_SHARDS) (
  shard_bytes[i] = platform_limits[shard_platform[i]]
);

% Objective: maximize total information
var int: total_bytes = sum(i in 1..NUM_SHARDS)(shard_bytes[i]);

solve maximize total_bytes;

% Output
output [
  "=== OPTIMAL 71-SHARD DISTRIBUTION ===\n\n",
  "Total Information: ", show(total_bytes), " bytes\n",
  "Average per shard: ", show(total_bytes div NUM_SHARDS), " bytes\n\n"
];
