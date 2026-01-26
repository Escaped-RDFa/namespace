% Simple per-shard optimization
int: shard_id;  % Parameter: which shard to optimize

% Platform limits (bytes after compression)
array[1..10] of int: limits = [180, 1500, 3000, 50, 45000, 50000, 10000, 35000, 300, 240];

% Decision: which platform (1-10)
var 1..10: platform;

% Decision: bytes to use
var int: bytes;

% Constraint: fit in platform limit
constraint bytes = limits[platform];

% Objective: maximize bytes
solve maximize bytes;

output [show(platform), ",", show(bytes)];
