# csv-to-plinks

Read a CSV file, generate plinks and save them to another one.

## Usage

Your input csv file should follow the format:

```
item_ordered,name,amount_owed
Something Ordered,John Doe,100.00
Something Else,Jane Doe,90.00
```

You can run `csv-to-plinks` as follows:

```
csv-to-plinks run -i /path/to/input.csv -o /path/to/output.csv --api-key {your mollie api key}
```

Make sure that `input.csv` exists, and `output.csv` doesn't, and that you provide a valid API key.

The format of `output.csv` will be:

```
Name,Amount,Payment Link
John Doe,100.00,{plink}
Jane Doe,90.00,{plink}
```

You can check the status of payment requests created by this program by using the `check` command:

```
csv-to-plinks check -i /path/to/output.csv --api-key {your mollie api key}
```

This will go through the CSV that was generated by the `run` command, and check if the payment requests have been paid against the Mollie API.