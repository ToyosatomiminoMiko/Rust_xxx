create table if not exists `test_table`
(
`id` int unsigned  not null primary key,
`username` varchar(256) not null ,
`info` varchar(256) not null ,
`group` int unsigned  not null
);
-- testing
INSERT INTO test_table VALUES (0, 'tempest', 'what can i say', 0);

select * from test_table;
