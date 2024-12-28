require 'sinatra'
require 'jwt'
require 'json'

hmac_secret = ENV['JWT_KEY']

before '/api/*' do
    auth_header = request.env['HTTP_AUTHORIZATION']
    raise StandardError.new 'No auth header found' if auth_header.nil?

    @key = JWT.decode(auth_header, hmac_secret, true, { algorithm: 'HS256' })
end

get '/api/hello' do
    content_type :json

    { msg: 'Hello World' }.to_json
end

get '/token' do
    content_type :json
    status 200
    {
        token: JWT.encode({role: 'admin'}, hmac_secret, 'HS256'),
    }.to_json
end
