# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name          = 'hedl'
  spec.version       = '1.0.0'
  spec.authors       = ['Dweve']
  spec.email         = ['contact@dweve.com']

  spec.summary       = 'HEDL (Hierarchical Entity Data Language) Ruby bindings'
  spec.description   = 'Token-efficient data format optimized for LLM context windows. ' \
                       'Provides 5-10x compression compared to JSON.'
  spec.homepage      = 'https://github.com/dweve-ai/hedl'
  spec.license       = 'Apache-2.0 OR MIT'

  spec.required_ruby_version = '>= 2.7.0'

  spec.files         = Dir['lib/**/*.rb', 'README.md', 'LICENSE']
  spec.require_paths = ['lib']

  spec.add_dependency 'ffi', '~> 1.15'

  spec.metadata['homepage_uri'] = spec.homepage
  spec.metadata['source_code_uri'] = 'https://github.com/dweve-ai/hedl'
  spec.metadata['changelog_uri'] = 'https://github.com/dweve-ai/hedl/blob/main/CHANGELOG.md'
end
