def process(items)
  items.each { |item| handle(item) }
  items.map { |i| i.name }
  items.select { |i| i.active? }
end
