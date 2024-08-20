import requests
import json
import argparse
import csv
from urllib3.exceptions import InsecureRequestWarning

# Suppress only the single InsecureRequestWarning from urllib3 needed for self-signed certs
requests.packages.urllib3.disable_warnings(category=InsecureRequestWarning)


def _request_call(method, url, headers, data):
    try:
        response = requests.request(
            method, url, headers=headers, data=data, verify=False
        )  # Disable SSL verification
        response.raise_for_status()
        return response
    except requests.exceptions.RequestException as e:
        print(f"Error during request: {e}")
        return None


def get_tags(canary, api_version, api_token, application, timezone):
    url = f"{canary}/{api_version}/browseTags"
    payload = json.dumps(
        {
            "application": application,
            "timezone": timezone,
            "apiToken": api_token,
            "path": "",
            "deep": True,
            "search": "",
        }
    )
    headers = {"Content-Type": "application/json"}

    response = _request_call("POST", url, headers, data=payload)
    if response:
        return response.json().get("tags", [])
    return []


def get_tag_context(canary, api_version, api_token, tags):
    url = f"{canary}/{api_version}/getTagContext"
    payload = json.dumps({"apiToken": api_token, "tags": tags})
    headers = {"Content-Type": "application/json"}

    response = _request_call("POST", url, headers, data=payload)
    if response:
        return response.json().get("data", [])
    return []


def save_to_csv(data, filename):
    with open(filename, mode="w", newline="") as file:
        writer = csv.DictWriter(
            file,
            fieldnames=[
                "tagName",
                "historianItemId",
                "sourceItemId",
                "oldestTimeStamp",
                "latestTimeStamp",
            ],
        )
        writer.writeheader()
        for item in data:
            tag_context = item["tagContext"]
            writer.writerow(
                {
                    "tagName": item["tagName"],
                    "historianItemId": tag_context.get("historianItemId"),
                    "sourceItemId": tag_context.get("sourceItemId"),
                    "oldestTimeStamp": tag_context.get("oldestTimeStamp"),
                    "latestTimeStamp": tag_context.get("latestTimeStamp"),
                }
            )


def save_to_txt(data, filename):
    with open(filename, mode="w") as file:
        for item in data:
            file.write(f"TagName: {item['tagName']}\n")
            file.write(
                f"  HistorianItemId: {item['tagContext'].get('historianItemId')}\n"
            )
            file.write(f"  SourceItemId: {item['tagContext'].get('sourceItemId')}\n")
            file.write(
                f"  OldestTimeStamp: {item['tagContext'].get('oldestTimeStamp')}\n"
            )
            file.write(
                f"  LatestTimeStamp: {item['tagContext'].get('latestTimeStamp')}\n"
            )
            file.write("\n")


def save_to_json(data, filename):
    with open(filename, mode="w") as file:
        json.dump(data, file, indent=4)


def main(
    canary, api_version, api_token, application, timezone, output_format, output_file
):
    tags = get_tags(canary, api_version, api_token, application, timezone)
    if tags:
        tag_context_data = get_tag_context(canary, api_version, api_token, tags)
        if tag_context_data:
            if output_format == "csv":
                save_to_csv(tag_context_data, output_file)
            elif output_format == "txt":
                save_to_txt(tag_context_data, output_file)
            elif output_format == "json":
                save_to_json(tag_context_data, output_file)
            print(f"Data saved to {output_file} in {output_format} format.")
        else:
            print("No data returned in the tag context.")
    else:
        print("No tags found.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="CLI tool to interact with the Canary API."
    )
    parser.add_argument(
        "--canary", type=str, required=True, help="Base URL for the Canary server"
    )
    parser.add_argument(
        "--api_version", type=str, default="api/v2", help="API version to use"
    )
    parser.add_argument(
        "--api_token", type=str, required=True, help="API token for authentication"
    )
    parser.add_argument(
        "--application", type=str, default="Postman Test", help="Application name"
    )
    parser.add_argument(
        "--timezone", type=str, default="Pacific Standard Time", help="Timezone to use"
    )
    parser.add_argument(
        "--output_format",
        type=str,
        choices=["csv", "txt", "json"],
        required=True,
        help="Output format for saving the data",
    )
    parser.add_argument(
        "--output_file", type=str, required=True, help="Output file name"
    )

    args = parser.parse_args()

    main(
        args.canary,
        args.api_version,
        args.api_token,
        args.application,
        args.timezone,
        args.output_format,
        args.output_file,
    )
