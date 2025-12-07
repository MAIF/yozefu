 from end - 5_000
where value contains "uv"
  and k contains "foobar"
   or t == "french-recipes"
  and !(partition != 1)
order by timestamp asc
limit 100