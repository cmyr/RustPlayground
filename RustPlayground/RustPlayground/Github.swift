//
//  Github.swift
//  RustPlayground
//
//  Created by Colin Rofls on 2019-05-05.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Foundation

fileprivate let HTTP_HEADER_AUTH = "Authorization"
fileprivate let HTTP_HEADER_GITHUB_SCOPES = "X-OAuth-Scopes"
fileprivate let HTTP_HEADER_LOCATION = "Location"

typealias GistIdentifier = String

extension GistIdentifier {
    func toGistUrl() -> URL {
        let baseUrl = URL(string: "https://gist.github.com")!
        return baseUrl.appendingPathComponent(self)
    }

    func toPlaygroundUrl() -> URL {
        var urlComponents = URLComponents()
        urlComponents.percentEncodedPath = "https://play.rust-lang.org/"
        urlComponents.queryItems = [
            URLQueryItem(name: "version", value: "stable"),
            URLQueryItem(name: "mode", value: "debug"),
            URLQueryItem(name: "edition", value: "2018"),
            URLQueryItem(name: "gist", value: self),
        ]
        return urlComponents.url!
    }
}

enum GithubError {
    /// The request returned an error
    case Connection(Error)
    case InvalidResponse
    case NotAuthorized
    /// The token does not have the 'gists' authorization
    case MissingGistAuthorization
    /// The request returned okay, but was missing an expected header
    case MissingExpectedHeader
    case UnexpectedStatus(Int)
}

extension GithubError: Error {
    var localizedDescription: String {
        switch self {
        case .Connection(let error):
            return "Failed to connect to github.com. '\(error.localizedDescription)'"
        case .InvalidResponse:
            return "Authorization failed, invalid response."
        case .NotAuthorized:
            return "Authorization failed, invalid token. You can update it in the application preferences."
        case .MissingGistAuthorization:
            return "Authorization failed. The provided token does not have the 'gists' permission."
        case .MissingExpectedHeader:
            return "Authorization failed, missing expected header. Please open a bug report."
        case .UnexpectedStatus(let code):
            return "Post failed, received status \(code)"
        }
    }
}

class GithubConnection {
    let token: String
    let baseUrl = URL(string: "https://api.github.com")!

    init(token: String) {
        self.token = token
    }

    func validate(completionHandler: @escaping (GithubError?) -> ()) {
        let url = baseUrl.appendingPathComponent("user")
        var request = URLRequest(url: url)
        request.addValue("token \(token)", forHTTPHeaderField: HTTP_HEADER_AUTH)
        URLSession.shared.dataTask(with: request) { (data, response, error) in
            DispatchQueue.main.async {
                if let error = error {
                    return completionHandler(.Connection(error))
                }

                guard let response = response as? HTTPURLResponse else {
                    return completionHandler(.InvalidResponse)
                }

                if response.statusCode != 200 {
                    return completionHandler(.NotAuthorized)
                }

                guard let authorizedScopes = response.value(forHTTPHeaderField: HTTP_HEADER_GITHUB_SCOPES) else {
                    return completionHandler(.MissingExpectedHeader)
                }

                if !authorizedScopes.contains("gist") {
                    return completionHandler(.MissingGistAuthorization)
                }

                completionHandler(nil)
            }
        }.resume()
    }

    func createGist(withContent content: String, completionHandler: @escaping (Result<GistIdentifier, GithubError>) -> ()) {
        let url = baseUrl.appendingPathComponent("gists")
        var request = URLRequest(url: url)
        request.addValue("token \(token)", forHTTPHeaderField: HTTP_HEADER_AUTH)
        request.httpMethod = "POST"

        let fileObject: [String: [String: AnyObject]] = ["playground.rs": ["content": content as AnyObject]]
        let json: [String: AnyObject] = ["files": fileObject as AnyObject]

        do {
            let jsonData = try JSONSerialization.data(withJSONObject: json)
            request.httpBody = jsonData
            URLSession.shared.dataTask(with: request) { (data, response, error) in
                if let error = error {
                    return completionHandler(.failure(.Connection(error)))
                }

                guard let response = response as? HTTPURLResponse else {
                    return completionHandler(.failure(.InvalidResponse))
                }

                if response.statusCode != 201 {
                    return completionHandler(.failure(.NotAuthorized))
                }

                guard let location = response.allHeaderFields[HTTP_HEADER_LOCATION] as? String else {
                    return completionHandler(.failure(.MissingExpectedHeader))
                }

                let gistId = location.split(separator: "/").last!
                completionHandler(.success(String(gistId)))
            }.resume()
        } catch {
            print("this didn't happen")
        }
    }
}
