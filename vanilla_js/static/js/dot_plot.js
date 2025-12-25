function drawDotPlot(plot_identifier, in_data) {
    // Set Dimensions
    const xSize = 1500;
    const ySize = 1000;
    const margin = 40;
    const xMax = xSize - margin*2;
    const yMax = ySize - margin*2;

    const data = [];

    console.log(data)
    
    var time_min = Infinity
    var time_max = 0
    var distance_min = Infinity
    var distance_max = 0
    in_data.forEach(e => {
        if (e.time > time_max) {
            time_max = e.time
        }
        if (e.time < time_min) {
            time_min = e.time
        }
        if (e.distance > distance_max) {
            distance_max = e.distance
        }
        if (e.distance < distance_min) {
            distance_min = e.distance
        }
        data.push(
            [e.time, e.distance]
        )
    });

    console.log(data)

    // Append SVG Object to the Page
    const svg = d3.select(plot_identifier)
      .append("svg")
      .append("g")
      .attr("transform","translate(" + margin + "," + margin + ")");

    // X Axis
    const x = d3.scaleTime()
      .domain([time_min, time_max])
      .range([0, xMax]); // xMax

    svg.append("g")
      .attr("transform", "translate(0," + yMax + ")")
      .call(d3.axisBottom(x));

    // Y Axis
    const y = d3.scaleLinear()
      .domain([distance_min, distance_max])
      .range([ yMax, 0]); // yMax

    svg.append("g")
      .call(d3.axisLeft(y));

    // Dots
    svg.append('g')
      .selectAll("dot")
      .data(in_data).enter()
      .append("circle")
      .attr("cx", function (d) { return x(d.time) } )
      .attr("cy", function (d) { return y(d.distance) } )
      .attr("r", 10)
      .style("fill", "Red")
      .on("mouseover", function(d) {
          d3.select(this).attr("r", 15).style("fill", "blue");
          console.log(d3.select(this).attr("cx"), d3.select(this).attr("cy") )
        })                  
        .on("mouseout", function(d) {
          d3.select(this).attr("r", 10).style("fill", "red");
        })
        .append("svg:title")
      .text(function(d, i) {
        return in_data[i].name
       });
    
    
}
