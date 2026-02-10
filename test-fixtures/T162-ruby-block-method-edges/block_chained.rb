def complex_transform(users)
  users.select { |u| u.active? }
       .map { |u| u.name }
       .compact
       .uniq
       .sort
end
