


function parseCSV(file) {

    return new Promise((resolve, reject) => {
        Papa.parse(file, {
            header: false,
            skipEmptyLines: true,
            complete: (results) => {
                const rows = results.data;

                if (rows.length <= 1) {
                    messageDiv.innerHTML = `<p style="color:red;">CSV is empty or missing data rows.</p>`;
                    return;
                }

                // first 5 data rows
                const parser = d3.timeParse("%s");
                const first5 = rows.slice(2,1000);

                let data = first5.map((row) => {
                    const date = new Date(row[1]); // original string format
                    // const unixMs = date.getTime(); // convert to unix time (ms)

                    return {
                        time: date,                   
                        distance: parseFloat(row[6]),
                        name: row[2]
                    };
                });

                console.log("Parsed with Unix time:", data);

                document.getElementById('message').innerHTML =
                    `<p style="color:green;">Parsed & converted dates to Unix time!</p>`;

                return resolve(data)
            }
        });
    });
}