#!/usr/bin/env ruby
require 'json'

CT = "Content-Type: application/json"
BASE = "http://localhost:8998"

def new_user(name)
  data = JSON.generate({username: name})
  cmd = "curl -s -H '#{CT}' -X POST #{BASE}/users -d '#{data}'"
end

def find_user(name)
  cmd = "curl -s -H '#{CT}' #{BASE}/users/find/#{name}"
end

def get_user(id)
  cmd = "curl -s -H '#{CT}' #{BASE}/users/#{id}"
end

def create_post(id, title, body)
  data = JSON.generate({title: title, body: body})
  cmd = "curl -s -H '#{CT}' -X POST #{BASE}/users/#{id}/posts -d '#{data}'"
end

def all_posts
  cmd = "curl -s -H '#{CT}' #{BASE}/posts"
end

def users_posts(id)
  cmd = "curl -s -H '#{CT}' #{BASE}/users/#{id}/posts"
end

def publish_post(id)
  cmd = "curl -s -H '#{CT}' -X POST #{BASE}/posts/#{id}/publish"
end

def add_comment(id, user_id, body)
  data = JSON.generate({user_id: user_id, body: body})
  cmd = "curl -s -H '#{CT}' -X POST #{BASE}/posts/#{id}/comments -d '#{data}'"
end

def users_comments(id)
  cmd = "curl -s -H '#{CT}' #{BASE}/users/#{id}/comments"
end

def posts_comments(id)
  cmd = "curl -s -H '#{CT}' #{BASE}/posts/#{id}/comments"
end

def execute(description, cmd)
  puts "## #{description}"
  puts cmd
  result = `#{cmd}`
  puts JSON.pretty_generate(JSON.parse(result))
  puts
end

def run
  [
    ["Create a new user", new_user("Frank")],
    ["Create a new user", new_user("Bob")],
    ["Create a new user already exists", new_user("Bob")],
    ["Lookup user by name", find_user("Frank")],
    ["Lookup user by name that doesn't exist", find_user("Steve")],
    ["Create a post", create_post(1, "Frank says hello", "Hello friends")],
    ["Create a post", create_post(2, "Bob is here too", "Hello friends, also")],
    ["Publish a post", publish_post(1)],
    ["Comment on a post", add_comment(1, 2, "Hi Frank, this is your friend Bob")],
    ["List all posts", all_posts],
    ["See posts", users_posts(1)],
    ["Publish other post", publish_post(2)],
    ["List all posts again", all_posts],
    ["See users comments", users_comments(2)],
    ["See post comments", posts_comments(1)],
  ].each do |description, cmd|
    execute(description, cmd)
  end
end

run
