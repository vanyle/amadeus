import {
  Button,
  FormControl,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  TextField,
} from "@material-ui/core";
import { Autocomplete } from "@mui/material";
import { AdapterDateFns } from "@mui/x-date-pickers/AdapterDateFnsV3";
import { DatePicker } from "@mui/x-date-pickers/DatePicker/DatePicker";
import { LocalizationProvider } from "@mui/x-date-pickers/LocalizationProvider/LocalizationProvider";
import * as datefns from "date-fns";
import React, { useCallback, useState } from "react";
import {
  VictoryAxis,
  VictoryChart,
  VictoryLegend,
  VictoryLine,
  VictoryTheme,
} from "victory";
import "./App.css";
import { airports } from "./airports";

type DataFetchParams = {
  OnD: string;
  trip_type: string;
  search_date_min: string;
  search_date_max: string;
};

const fetchData = async ({
  OnD,
  trip_type,
  search_date_max,
  search_date_min,
}: DataFetchParams) => {
  try {
    let currentUrl = location.href;
    let request = await fetch(
      `${currentUrl}?OnD=${OnD}&trip_type=${trip_type}&search_date_min=${search_date_min}&search_date_max=${search_date_max}`
    );

    return await request.json();
  } catch (err) {
    console.log("Failed to fetch: ", err);
  }
};

const randomColor = () => {
  return "#" + Math.floor(Math.random() * 16777215).toString(16);
};

type DataState = {
  airline: string;
  lineColor: string;
  prices: { advance_purchase: number; price: number }[];
}[];

function App() {
  const [data, setData] = useState<DataState>([]);
  const [tripType, setTripType] = useState("RT");
  const [whereFrom, setWhereFrom] = useState("PAR");
  const [whereTo, setWhereTo] = useState("LIS");

  const [searchDateMax, setSearchDateMax] = useState<Date>(
    datefns.parse("2025/11/17", "yyyy/MM/dd", new Date())
  );
  const [searchDateMin, setSearchDateMin] = useState<Date>(
    datefns.parse("2020/11/17", "yyyy/MM/dd", new Date())
  );

  const makeSearch = useCallback(async () => {
    const OnD = `${whereFrom}-${whereTo}`;
    const searchDateMaxStr = datefns.format(searchDateMax, "yyyy/MM/dd");
    const searchDateMinStr = datefns.format(searchDateMin, "yyyy/MM/dd");

    let data = await fetchData({
      OnD: OnD,
      search_date_max: searchDateMaxStr,
      search_date_min: searchDateMinStr,
      trip_type: tripType,
    });
    let airlines: string[] = [];
    for (let dataPoint of data) {
      if (airlines.includes(dataPoint.airline)) {
        continue;
      }
      airlines.push(dataPoint.airline);
    }

    console.log(
      "fetch parameters: ",
      OnD,
      searchDateMaxStr,
      searchDateMinStr,
      tripType
    );
    console.log(data);

    let result: DataState = [];
    for (let airline of airlines) {
      let prices = data
        .filter((d) => d.airline === airline)
        .map((d) => ({
          advance_purchase: d.advance_purchase,
          price: d.price,
        }));
      result.push({ airline, prices, lineColor: randomColor() });
    }

    setData(result);
  }, [setData, tripType, whereFrom, whereTo, searchDateMax, searchDateMin]);

  return (
    <div className="flex flex-col items-center">
      <h1 className="text-amadeus font-bold">Amadeus analytics</h1>
      <Paper className="p-6 flex flex-col items-start">
        <h2 className="text-2xl text-left font-bold">
          Search for flight prices
        </h2>
        <div className="h-4" />
        <div className="flex justify-center">
          <FormControl className="w-[200px]">
            <InputLabel>Trip type</InputLabel>
            <Select
              labelId="demo-simple-select-label"
              id="demo-simple-select"
              value={tripType}
              label="Trip type"
              onChange={(evt) => {
                setTripType(evt.target.value === "RT" ? "RT" : "OW");
              }}
            >
              <MenuItem value={"OW"}>One way</MenuItem>
              <MenuItem value={"RT"}>Round trip</MenuItem>
            </Select>
          </FormControl>
          <div className="w-4 inline-block" />
          <Autocomplete
            className="inline"
            options={airports}
            sx={{ width: 300 }}
            value={whereFrom}
            renderInput={(params) => (
              <TextField
                {...params}
                variant="outlined"
                onChange={(evt) => {
                  setWhereFrom(evt.target.value);
                }}
                label="Where from?"
              />
            )}
          />
          <div className="w-4 inline-block" />

          <Autocomplete
            options={airports}
            sx={{ width: 300 }}
            value={whereTo}
            renderInput={(params) => (
              <TextField
                {...params}
                variant="outlined"
                onChange={(evt) => {
                  setWhereTo(evt.target.value);
                }}
                label="Where to?"
              />
            )}
          />
          <div className="w-4 inline-block" />

          <LocalizationProvider dateAdapter={AdapterDateFns}>
            <DatePicker
              value={searchDateMin}
              onChange={(newValue) => {
                setSearchDateMin(newValue ?? new Date());
              }}
              label="Search for trips after "
            />
            <div className="w-4 inline-block" />

            <DatePicker
              value={searchDateMax}
              onChange={(newValue) => setSearchDateMax(newValue ?? new Date())}
              label="Search for trips before"
            />
          </LocalizationProvider>
        </div>
        <div className="h-4" />
        <div className="text-left">
          <Button onClick={makeSearch} variant="contained" color="primary">
            Explore
          </Button>
        </div>
      </Paper>

      <div className="w-[100%] flex flex-col">
        <div className="text-xl text-left p-5">
          Cheapest median price: {100}€
        </div>
        <VictoryChart width={700} height={400} theme={VictoryTheme.material}>
          <VictoryLegend
            x={0}
            y={0}
            centerTitle
            orientation="horizontal"
            gutter={20}
            style={{ border: { stroke: "black" }, title: { fontSize: 20 } }}
            data={data.map((d) => ({
              name: d.airline,
              symbol: { fill: d.lineColor },
            }))}
          />
          {data.map((d) => {
            return (
              <VictoryLine
                key={d.airline}
                style={{
                  data: { stroke: d.lineColor },
                  parent: { border: "1px solid #ccc" },
                }}
                data={d.prices.map((p) => ({
                  x: p.advance_purchase,
                  y: p.price,
                }))}
              />
            );
          })}
          <VictoryAxis
            label="Advance purchase (in days)"
            style={{
              tickLabels: { fontSize: 15, padding: 5 },
              ticks: { stroke: "grey", size: 5 },
              axisLabel: { fontSize: 20, padding: 30 },
            }}
          />
          <VictoryAxis
            dependentAxis
            label="Ticket price (in euros)"
            style={{ axisLabel: { fontSize: 15, padding: 30 } }}
          />
        </VictoryChart>
      </div>
    </div>
  );
}

export default App;
