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

enum GithubError {
    /// The request returned an error
    case Connection(Error)
    case InvalidResponse
    case NotAuthorized
    /// The token does not have the 'gists' authorization
    case MissingGistAuthorization
    /// The request returned okay, but was missing an expected header
    case MissingExpectedHeader
}

extension GithubError: Error {
    var localizedDescription: String {
        switch self {
        case .Connection(let error):
            return "Failed to connect to github.com. '\(error.localizedDescription)'"
        case .InvalidResponse:
            return "Authorization failed, invalid response."
        case .NotAuthorized:
            return "Authorization failed, invalid token."
        case .MissingGistAuthorization:
            return "Authorization failed. The provided token does not have the 'gists' permission."
        case .MissingExpectedHeader:
            return "Authorization failed, missing expected header. Please open a bug report."
        }
    }
}

class GithubConnection {
    let username: String
    let token: String
    let baseUrl = URL(string: "https://api.github.com")!

    init(username: String, token: String) {
        self.username = username
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

                guard let authorizedScopes = response.allHeaderFields[HTTP_HEADER_GITHUB_SCOPES] as? String else {
                    return completionHandler(.MissingExpectedHeader)
                }

                if !authorizedScopes.contains("gist") {
                    return completionHandler(.MissingGistAuthorization)
                }

                completionHandler(nil)
            }
        }.resume()
    }
}
