def transform(data)
  data.each do |item|
    process(item)
  end

  data.select do |d|
    d.valid?
  end
end
